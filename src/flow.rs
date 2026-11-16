//! Which Credential Manager request a sign-in sends, and what follows each answer.

use crate::config::Mode;
use crate::error::{ErrorCode, SignInError};

/// One Credential Manager request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Attempt {
    /// A `GetGoogleIdOption` request.
    GoogleId {
        /// Offer only accounts that already signed in to this app.
        filter_by_authorized_accounts: bool,
        /// Sign in without a sheet when exactly one account matches.
        auto_select: bool,
    },
    /// A `GetSignInWithGoogleOption` request, the flow behind a sign-in button.
    SignInWithGoogle,
}

/// What to do after the answer to an [`Attempt`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Next {
    /// Send this request.
    Request(Attempt),
    /// The sign-in failed.
    Fail(SignInError),
}

/// The state of the sign-in, one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flow {
    running: Option<(Mode, Attempt)>,
}

const AUTHORIZED_ONLY: Attempt = Attempt::GoogleId {
    filter_by_authorized_accounts: true,
    auto_select: true,
};

const EVERY_ACCOUNT: Attempt = Attempt::GoogleId {
    filter_by_authorized_accounts: false,
    auto_select: false,
};

impl Flow {
    /// Starts a sign-in and gives its first request.
    ///
    /// # Errors
    ///
    /// [`ErrorCode::Busy`] while another sign-in runs.
    ///
    /// ```
    /// use hqf_godot_google_signin::Mode;
    /// use hqf_godot_google_signin::flow::{Attempt, Flow};
    /// let mut flow = Flow::default();
    /// assert_eq!(flow.start(Mode::Button), Ok(Attempt::SignInWithGoogle));
    /// assert!(flow.start(Mode::Button).is_err());
    /// ```
    pub fn start(&mut self, mode: Mode) -> Result<Attempt, SignInError> {
        if self.running.is_some() {
            return Err(SignInError::new(
                ErrorCode::Busy,
                "a sign-in is already running",
            ));
        }
        let attempt = match mode {
            Mode::Automatic | Mode::Silent => AUTHORIZED_ONLY,
            Mode::AccountChooser => EVERY_ACCOUNT,
            Mode::Button => Attempt::SignInWithGoogle,
        };
        self.running = Some((mode, attempt));
        Ok(attempt)
    }

    /// The request the running sign-in waits on.
    pub fn current(&self) -> Option<Attempt> {
        self.running.map(|(_, attempt)| attempt)
    }

    /// Whether a sign-in runs.
    pub const fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// The running request failed with `error`: gives the request to send next, or the
    /// failure to report.
    ///
    /// Only [`Mode::Automatic`] falls back, from the accounts that already signed in to
    /// every account, and only when there were none
    /// ([`ErrorCode::NoCredential`]): a player who closed the sheet is not shown another.
    /// An error with no sign-in running gives [`ErrorCode::Bridge`].
    pub fn fail(&mut self, error: SignInError) -> Next {
        let Some((mode, attempt)) = self.running.take() else {
            return Next::Fail(SignInError::new(
                ErrorCode::Bridge,
                format!("answer with no sign-in running: {error}"),
            ));
        };
        if mode == Mode::Automatic
            && attempt == AUTHORIZED_ONLY
            && error.code == ErrorCode::NoCredential
        {
            self.running = Some((mode, EVERY_ACCOUNT));
            return Next::Request(EVERY_ACCOUNT);
        }
        Next::Fail(error)
    }

    /// The running request succeeded; the sign-in is over.
    ///
    /// # Errors
    ///
    /// [`ErrorCode::Bridge`] when no sign-in was running.
    pub fn succeed(&mut self) -> Result<(), SignInError> {
        self.running
            .take()
            .map(|_| ())
            .ok_or_else(|| SignInError::new(ErrorCode::Bridge, "answer with no sign-in running"))
    }

    /// Forgets the running sign-in, if any.
    pub const fn cancel(&mut self) {
        self.running = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(code: ErrorCode) -> SignInError {
        SignInError::new(code, "x")
    }

    #[test]
    fn first_request_of_each_mode() {
        let cases = [
            (Mode::Automatic, AUTHORIZED_ONLY),
            (Mode::Silent, AUTHORIZED_ONLY),
            (Mode::AccountChooser, EVERY_ACCOUNT),
            (Mode::Button, Attempt::SignInWithGoogle),
        ];
        for (mode, attempt) in cases {
            let mut flow = Flow::default();
            assert!(!flow.is_running());
            assert_eq!(flow.start(mode), Ok(attempt), "{mode:?}");
            assert!(flow.is_running());
            assert_eq!(flow.current(), Some(attempt));
        }
    }

    #[test]
    fn second_start_is_busy_and_keeps_the_first() {
        let mut flow = Flow::default();
        flow.start(Mode::Silent).unwrap();
        let busy = flow.start(Mode::Button).unwrap_err();
        assert_eq!(busy.code, ErrorCode::Busy);
        assert_eq!(flow.current(), Some(AUTHORIZED_ONLY));
    }

    #[test]
    fn automatic_falls_back_once_on_no_credential() {
        let mut flow = Flow::default();
        flow.start(Mode::Automatic).unwrap();
        assert_eq!(
            flow.fail(error(ErrorCode::NoCredential)),
            Next::Request(EVERY_ACCOUNT)
        );
        assert_eq!(flow.current(), Some(EVERY_ACCOUNT));
        let last = error(ErrorCode::NoCredential);
        assert_eq!(flow.fail(last.clone()), Next::Fail(last));
        assert!(!flow.is_running());
    }

    #[test]
    fn no_fallback_after_any_other_error() {
        for code in ErrorCode::ALL {
            if code == ErrorCode::NoCredential {
                continue;
            }
            let mut flow = Flow::default();
            flow.start(Mode::Automatic).unwrap();
            assert_eq!(flow.fail(error(code)), Next::Fail(error(code)), "{code}");
            assert!(!flow.is_running());
        }
    }

    #[test]
    fn other_modes_never_fall_back() {
        for mode in [Mode::Silent, Mode::AccountChooser, Mode::Button] {
            let mut flow = Flow::default();
            flow.start(mode).unwrap();
            let no = error(ErrorCode::NoCredential);
            assert_eq!(flow.fail(no.clone()), Next::Fail(no), "{mode:?}");
            assert!(!flow.is_running());
        }
    }

    #[test]
    fn success_ends_the_sign_in() {
        let mut flow = Flow::default();
        flow.start(Mode::Button).unwrap();
        assert_eq!(flow.succeed(), Ok(()));
        assert!(!flow.is_running());
        assert_eq!(flow.succeed().unwrap_err().code, ErrorCode::Bridge);
        assert_eq!(flow.start(Mode::Button), Ok(Attempt::SignInWithGoogle));
    }

    #[test]
    fn answers_without_a_sign_in() {
        let mut flow = Flow::default();
        let Next::Fail(failure) = flow.fail(error(ErrorCode::Cancelled)) else {
            panic!("expected a failure");
        };
        assert_eq!(failure.code, ErrorCode::Bridge);
        assert_eq!(
            failure.message,
            "answer with no sign-in running: cancelled: x"
        );
    }

    #[test]
    fn cancel_frees_the_flow() {
        let mut flow = Flow::default();
        flow.start(Mode::AccountChooser).unwrap();
        flow.cancel();
        assert!(!flow.is_running());
        assert_eq!(flow.current(), None);
        assert!(flow.start(Mode::Silent).is_ok());
    }
}
