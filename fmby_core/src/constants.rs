pub const FMHY_SINGLE_PAGE_ENDPOINT: &str = "https://api.fmhy.net/single-page";
pub const AUTO_THREAD_MAPPINGS: &[(u64, Option<&str>)] = &[
    (FmhyChannel::FREE_STUFF, Some("956006107564879873")),
    (FmhyChannel::SECRET_RESOURCES, None),
];

pub struct DevChannel;

impl DevChannel {
    pub const ADD_LINKS: u64 = 1407275510769258567;
    pub const LINK_TESTING: u64 = 1407275689127837726;
    pub const REMOVE_SITES: u64 = 1407281582221295746;
    pub const DEAD_SITES: u64 = 1407281564248838175;
    pub const TESTING: u64 = 1410279582124347452;
}

pub struct FmhyChannel;

impl FmhyChannel {
    pub const ADD_LINKS: u64 = 997291314389467146;
    pub const LINK_TESTING: u64 = 1250924744853819547;
    pub const REMOVE_SITES: u64 = 986617857133649921;
    pub const DEAD_SITES: u64 = 988133247575810059;
    pub const RECENTLY_ADDED: u64 = 997012109156167710;
    pub const TODO: u64 = 997040018604433479;
    pub const NSFW_ADD_LINKS: u64 = 997292029056925808;
    pub const NSFW_REMOVED: u64 = 1136688501514047498;
    pub const NSFW_RECENTLY_ADDED: u64 = 1199379440292085901;
    pub const FREE_STUFF: u64 = 976770662205104150;
    pub const SECRET_RESOURCES: u64 = 1089999083856470016;
}

pub struct FmhyServerRole;

impl FmhyServerRole {
    pub const PRIVATEER: u64 = 1166287715524943912;
    pub const BOOSTER: u64 = 974702070508691597;
    pub const ATLANTEAN: u64 = 956006107564879876;
    pub const PIRATE: u64 = 956006107564879878;
    pub const FIRST_MATE: u64 = 956006107564879880;
    pub const FM_CHAT_MOD: u64 = 1250583645631156285;
    pub const CELESTIAL: u64 = 1195383987347140658;
    pub const CAPTAIN: u64 = 956006107577454603;
}

pub struct FmhyFeedRole;

impl FmhyFeedRole {
    pub const FREE_STUFF: u64 = 956006107564879873;
    pub const CANVA_INVITES: u64 = 1222863296529694751;
    pub const GAME_NIGHTS: u64 = 1100130880527290428;
    pub const SITE_UPDATES: u64 = 1169926358147813436;
    pub const WIKI_POLLS: u64 = 1198651940507238521;
    pub const KAI_POLLS: u64 = 1204590431933964298;
    pub const CAPTURE_FREE_STUFF: u64 = 1295437463837212745;
}

pub struct FmhyMiscRole;

impl FmhyMiscRole {
    pub const NSFW: u64 = 1195247836846108842;
    pub const EPIC_GAMER: u64 = 1191092193188909066;
    pub const AOC: u64 = 0;
}

pub mod link_testing {
    pub struct ForumTag;

    impl ForumTag {
        pub const ADDED: u64 = 1252866356546900091;
        pub const REJECTED: u64 = 1255077991566803038;
    }
}
