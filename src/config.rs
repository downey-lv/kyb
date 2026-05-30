pub struct KybConfig;

impl KybConfig {
    pub const SERVER_ADDRES: &str = "127.0.0.1:10001";
    pub const UPDATE_HOUR: u32 = 2;
    pub const UPDATE_MINUTE: u32 = 0;
    pub const DB_FILE: &str = "kyb.db";
    pub const SOURCE_MEMBER_OF_BOARD: &'static str = "https://dati.ur.gov.lv/officers/officers.csv";
    pub const SOURCE_COMPANIES: &'static str = "https://dati.ur.gov.lv/register/register.csv";
    pub const SOURCE_PUBLIC_INSTITUTIONS: &'static str =
        "https://dati.ur.gov.lv/register/ppi_public_persons_institutions.csv";
    pub const SOURCE_VAT: &'static str = "https://data.gov.lv/dati/dataset/9a5eae1c-2438-48cf-854b-6a2c170f918f/resource/610910e9-e086-4c5b-a7ea-0a896a697672/download/pdb_pvnmaksataji_odata.csv";
    pub const SOURCE_PVD: &'static str =
        "https://pakalpojumi.pvd.gov.lv/lv/opendata_files/ipvd_object_opendata/download";
}
