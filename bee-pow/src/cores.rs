#[derive(Clone)]
pub struct Cores(pub(self) usize);

impl Cores {
    pub fn max() -> Self {
        Self(num_cpus::get())
    }
}

impl From<usize> for Cores {
    fn from(num_cores: usize) -> Self {
        let max_cores = num_cpus::get();
        if num_cores > max_cores {
            Self(max_cores)
        } else {
            Self(num_cores)
        }
    }
}

impl std::ops::Deref for Cores {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
