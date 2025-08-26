#![allow(unused)]
pub enum DevChannel {
    AddLinks,
    LinkTesting,
    RemoveSites,
    DeadSites,
}

impl DevChannel {
    pub fn id(&self) -> u64 {
        match self {
            DevChannel::AddLinks => 1407275510769258567,
            DevChannel::LinkTesting => 1407275689127837726,
            DevChannel::RemoveSites => 1407281582221295746,
            DevChannel::DeadSites => 1407281564248838175,
        }
    }
}

pub enum FmhyChannel {
    AddLinks,
    LinkTesting,
    RemoveSites,
    DeadSites,
    ToDo,
    NsfwAddLinks,
    NsfwRemoved,
}

impl FmhyChannel {
    pub fn id(&self) -> u64 {
        match self {
            FmhyChannel::AddLinks => 997291314389467146,
            FmhyChannel::LinkTesting => 1250924744853819547,
            FmhyChannel::RemoveSites => 986617857133649921,
            FmhyChannel::DeadSites => 988133247575810059,
            FmhyChannel::ToDo => 997040018604433479,
            FmhyChannel::NsfwAddLinks => 997292029056925808,
            FmhyChannel::NsfwRemoved => 1136688501514047498,
        }
    }
}

pub const FMHY_SINGLE_PAGE_ENDPOINT: &str = "https://api.fmhy.net/single-page";

pub mod link_testing {
    /// Tags for this forum channel
    pub enum ForumTag {
        Added,
        Rejected,
    }

    impl ForumTag {
        /// Returns the raw [`u64`] ID for this tag
        pub fn id(self) -> u64 {
            match self {
                ForumTag::Added => 1407275958280392724,
                ForumTag::Rejected => 1407275979314954280,
            }
        }
    }
}
