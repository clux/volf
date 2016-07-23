
pub enum Progress {
    /// PR failed tests (to distinguish from Ready/Pending state)
    ///
    /// This means that the PR is still approved at current head, but build failed.
    /// A retry is allowed without further approval.
    Failure(String),
    /// PR in its initial state
    ///
    /// Every PR gets shifted to Ready after a Push event
    Ready,
    /// PR is waiting to be tested
    ///
    /// This will be triggered at the next free slot in the queue at auto branch
    Pending,
    /// PR is currently testing
    ///
    /// This can succeed (and so disappear after merging),
    /// fail (and so move to Failure state),
    /// or simply time out after an hour (and move to Failure state).
    Testing,
}

impl Default for Progress {
    fn default() -> Progress { Progress::Ready }
}

#[derive(Default)]
pub struct Pull {
    /// The full owner/repo string
    repo: String,
    /// Title of PR
    title: String,
    /// The pull request number
    pub num: u64,
    /// The current state of the PR
    state: Progress,
    // TODO: need base and head sha for bookkeeping
    // TODO: priority / blocked / rollup
    /// Username of approver, if approved
    approver: Option<String>,
    /// Whether this is allowed to progress to testing
    blocked: bool,
    /// Whether this PR is unmergeable
    unmergeable: bool,
}

impl Pull {
    pub fn new(full_name: &str, num: u64, title: &str) -> Pull {
        Pull {
            repo: full_name.into(),
            num: num,
            title: title.into(),
            ..Default::default()
        }
    }
    pub fn approve(&mut self, approver: &str) -> bool {
        if self.blocked {
            false
        } else {
            self.approver = Some(approver.into());
            self.state = Progress::Pending;
            true
        }
    }
    pub fn unblock(&mut self) { self.blocked = false; }
    pub fn block(&mut self) {
        match self.state {
            Progress::Testing => {
                // too late - need to cancel builds to stop it
            }
            _ => self.blocked = true,
        }
    }

    pub fn retry(&mut self) -> bool {
        if let Progress::Failure(_) = self.state {
            self.state = Progress::Pending;
            true
        } else {
            false
        }
    }

    pub fn test(&mut self) {
        self.state = Progress::Testing;
        unimplemented!()
    }
}
// TODO: trait to Trigger builds?
// TODO: how to handle build results?


pub fn parse_commands(pr: &mut Pull, comment: String, user: String) {
    let cmds = comment.split_whitespace().into_iter().filter(|&w| {
        w == "r+" || w == "retry" // keep it simple for now
    }).collect::<Vec<_>>();

    for cmd in cmds {
        info!("{}#{} - {} cmd from {}", pr.repo, pr.num, cmd, user);
        match cmd.as_ref() {
            "r+" => {
                pr.approve(&user);
            },
            "retry" => {
                pr.retry();
            },
            _ => {},
        }
    }
}

pub fn queue() {
    // loop over Pull instances
    // when no builds testing:
    // trigger next build in line
    // need to keep track if a PR is mergeable
    unimplemented!()
}
