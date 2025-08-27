#![allow(unused)]
use crate::id_str_enum;

pub const FMHY_SINGLE_PAGE_ENDPOINT: &str = "https://api.fmhy.net/single-page";
pub const AUTO_THREAD_MAPPINGS: &[(u64, Option<&str>)] = &[(
    FmhyChannel::FreeStuff.id(),
    Some(FmhyFeedRole::FreeStuff.id_str()),
)];

pub enum DevChannel {
    AddLinks,
    LinkTesting,
    RemoveSites,
    DeadSites,
}

id_str_enum!(DevChannel {
    AddLinks => 1407275510769258567,
    LinkTesting => 1407275689127837726,
    RemoveSites => 1407281582221295746,
    DeadSites => 1407281564248838175,
});

pub enum FmhyChannel {
    AddLinks,
    LinkTesting,
    RemoveSites,
    DeadSites,
    ToDo,
    NsfwAddLinks,
    NsfwRemoved,
    FreeStuff,
}

id_str_enum!(FmhyChannel {
    AddLinks => 997291314389467146,
    LinkTesting => 1250924744853819547,
    RemoveSites => 986617857133649921,
    DeadSites => 988133247575810059,
    ToDo => 997040018604433479,
    NsfwAddLinks => 997292029056925808,
    NsfwRemoved => 1136688501514047498,
    FreeStuff => 976770662205104150,
});

pub enum FmhyServerRole {
    Privateer,
    Booster,
    Atlantean,
    Pirate,
    FirstMate,
    FmChatMod,
    Celestial,
    Captain,
}

id_str_enum!(FmhyServerRole {
    Privateer => 1166287715524943912,
    Booster => 974702070508691597,
    Atlantean => 956006107564879876,
    Pirate => 956006107564879878,
    FirstMate => 956006107564879880,
    FmChatMod => 1250583645631156285,
    Celestial => 1195383987347140658,
    Captain => 956006107577454603,
});

pub enum FmhyFeedRole {
    FreeStuff,
    CanvaInvites,
    GameNights,
    SiteUpdates,
    WikiPolls,
    KaiPolls,
    CaptureFreeStuff,
}

id_str_enum!(FmhyFeedRole {
    FreeStuff => 956006107564879873,
    CanvaInvites => 1222863296529694751,
    GameNights => 1100130880527290428,
    SiteUpdates => 1169926358147813436,
    WikiPolls => 1198651940507238521,
    KaiPolls => 1204590431933964298,
    CaptureFreeStuff => 1295437463837212745
});

pub enum FmhyMiscRole {
    Nsfw,
    EpicGamer,
    Aoc,
}

id_str_enum!(FmhyMiscRole {
    Nsfw => 1195247836846108842,
    EpicGamer => 1191092193188909066,
    Aoc => 0,
});

pub mod link_testing {
    use super::*;

    pub enum ForumTag {
        Added,
        Rejected,
    }

    id_str_enum!(ForumTag {
        Added => 1407275958280392724,
        Rejected => 1407275979314954280
    });
}
