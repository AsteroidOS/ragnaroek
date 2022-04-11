use std::{fs::File, io::Read, path::Path};

use super::Pit;
use test_case::test_case;

const PIT_PATH: &str = "./testdata/";

#[test_case("GT-I8190.pit" ; "GT-I8190.pit")]
#[test_case("GT-I9500.pit" ; "GT-I9500.pit")]
#[test_case("GT-I9505.pit" ; "GT-I9505.pit")]
#[test_case("PICASSO3_EUR_OPEN.pit" ; "PICASSO3_EUR_OPEN.pit")]
#[test_case("SCH-I545.pit" ; "SCH-I545.pit")]
#[test_case("SGH-I317.pit" ; "SGH-I317.pit")]
#[test_case("SM-A105F-DS.pit" ; "SM-A105F-DS.pit")]
#[test_case("SM-A105FN_Europe.pit" ; "SM-A105FN_Europe.pit")]
#[test_case("SM-A105G.pit" ; "SM-A105G.pit")]
#[test_case("SM-A105N_KOR_Open.pit" ; "SM-A105N_KOR_Open.pit")]
#[test_case("SM-A3050_China_Open.pit" ; "SM-A3050_China_Open.pit")]
#[test_case("SM-A305FN.pit" ; "SM-A305FN.pit")]
#[test_case("SM-A305F.pit" ; "SM-A305F.pit")]
#[test_case("SM-A305GN.pit" ; "SM-A305GN.pit")]
#[test_case("SM-A305G.pit" ; "SM-A305G.pit")]
#[test_case("SM-A305GT.pit" ; "SM-A305GT.pit")]
#[test_case("SM-A305_Japan_KDI.pit" ; "SM-A305_Japan_KDI.pit")]
#[test_case("SM-A305N_Korea.pit" ; "SM-A305N_Korea.pit")]
#[test_case("SM-A305YN.pit" ; "SM-A305YN.pit")]
#[test_case("SM-A8000.pit" ; "SM-A8000.pit")]
#[test_case("SM-G530H.pit" ; "SM-G530H.pit")]
#[test_case("SM-G800H_EUR_OPEN.pit" ; "SM-G800H_EUR_OPEN.pit")]
#[test_case("SM-G900A.pit" ; "SM-G900A.pit")]
#[test_case("SM-G900F_16GB.pit" ; "SM-G900F_16GB.pit")]
#[test_case("SM-G900F_32GB.pit" ; "SM-G900F_32GB.pit")]
#[test_case("SM-G900F_pit.pit" ; "SM-G900F_pit.pit")]
#[test_case("SM-G900H_16GB.pit" ; "SM-G900H_16GB.pit")]
#[test_case("SM-G900I_16GB.pit" ; "SM-G900I_16GB.pit")]
#[test_case("SM-G900M.pit" ; "SM-G900M.pit")]
#[test_case("SM-G900P_16GB.pit" ; "SM-G900P_16GB.pit")]
#[test_case("SM-G900T_16GB.pit" ; "SM-G900T_16GB.pit")]
#[test_case("SM-G900W8_16GB.pit" ; "SM-G900W8_16GB.pit")]
#[test_case("SM-G900W8_32GB.pit" ; "SM-G900W8_32GB.pit")]
#[test_case("SM-G901F.pit" ; "SM-G901F.pit")]
#[test_case("SM-G9250_CHN_HKTW.pit" ; "SM-G9250_CHN_HKTW.pit")]
#[test_case("SM-G925A.pit" ; "SM-G925A.pit")]
#[test_case("SM-G925F_EUR_OPEN_HIDDEN100M.pit" ; "SM-G925F_EUR_OPEN_HIDDEN100M.pit")]
#[test_case("SM-G925F_EUR_OPEN_HIDDEN150M.pit" ; "SM-G925F_EUR_OPEN_HIDDEN150M.pit")]
#[test_case("SM-G925F_EUR_OPEN_HIDDEN200M.pit" ; "SM-G925F_EUR_OPEN_HIDDEN200M.pit")]
#[test_case("SM-G925F_EUR_OPEN_HIDDEN240M.pit" ; "SM-G925F_EUR_OPEN_HIDDEN240M.pit")]
#[test_case("SM-G925F_EUR_OPEN_HIDDEN300M.pit" ; "SM-G925F_EUR_OPEN_HIDDEN300M.pit")]
#[test_case("SM-G925F_EUR_OPEN_HIDDEN60M.pit" ; "SM-G925F_EUR_OPEN_HIDDEN60M.pit")]
#[test_case("SM-G925F_EUR_OPEN.pit" ; "SM-G925F_EUR_OPEN.pit")]
#[test_case("SM-G925I.pit" ; "SM-G925I.pit")]
#[test_case("SM-G925K-L-S_KOR.pit" ; "SM-G925K-L-S_KOR.pit")]
#[test_case("SM-G925P.pit" ; "SM-G925P.pit")]
#[test_case("SM-G925R4_USC.pit" ; "SM-G925R4_USC.pit")]
#[test_case("SM-G925T.pit" ; "SM-G925T.pit")]
#[test_case("SM-G925V.pit" ; "SM-G925V.pit")]
#[test_case("SM-G925W8.pit" ; "SM-G925W8.pit")]
#[test_case("SM-G950F_EUR_OPEN.pit" ; "SM-G950F_EUR_OPEN.pit")]
#[test_case("SM-G965F_EUR_OPEN.pit" ; "SM-G965F_EUR_OPEN.pit")]
#[test_case("SM-J200BT.pit" ; "SM-J200BT.pit")]
#[test_case("SM-J200F.pit" ; "SM-J200F.pit")]
#[test_case("SM-J200G.pit" ; "SM-J200G.pit")]
#[test_case("SM-J200GU.pit" ; "SM-J200GU.pit")]
#[test_case("SM-J200H.pit" ; "SM-J200H.pit")]
#[test_case("SM-J200M.pit" ; "SM-J200M.pit")]
#[test_case("SM-J200Y.pit" ; "SM-J200Y.pit")]
#[test_case("SM-J500M.pit" ; "SM-J500M.pit")]
#[test_case("SM-N900_32GB.pit" ; "SM-N900_32GB.pit")]
#[test_case("SM-N9005_16GB.pit" ; "SM-N9005_16GB.pit")]
#[test_case("SM-N9005_32GB.pit" ; "SM-N9005_32GB.pit")]
#[test_case("SM-N900A_32G.pit" ; "SM-N900A_32G.pit")]
#[test_case("SM-N900P_32GB.pit" ; "SM-N900P_32GB.pit")]
#[test_case("SM-N900V_32G.pit" ; "SM-N900V_32G.pit")]
#[test_case("SM-N910F_32GB.pit" ; "SM-N910F_32GB.pit")]
#[test_case("SM-N910T_32GB.pit" ; "SM-N910T_32GB.pit")]
#[test_case("SM-N910V.pit" ; "SM-N910V.pit")]
#[test_case("SM-R732.pit" ; "SM-R732.pit")]
#[test_case("SM-T210.pit" ; "SM-T210.pit")]
#[test_case("SM-T210(R).pit" ; "SM-T210(R).pit")]
#[test_case("SM-T231.pit" ; "SM-T231.pit")]
#[test_case("SM-T320.pit" ; "SM-T320.pit")]
#[test_case("SM-T365.pit" ; "SM-T365.pit")]
#[test_case("SM-T531.pit" ; "SM-T531.pit")]
#[test_case("SM-T535.pit" ; "SM-T535.pit")]
#[test_case("SM-T550.pit" ; "SM-T550.pit")]
#[test_case("SPH-L720_16GB.pit" ; "SPH-L720_16GB.pit")]
#[test_case("X1Q_CHN_OPENX.pit" ; "X1Q_CHN_OPENX.pit")]
#[test_case("X1Q_USA_SINGLE.pit" ; "X1Q_USA_SINGLE.pit")]
fn deserialize(file: &str) {
    // Enumerate all PIT files we have in the test directory
    let pit_path = Path::new(PIT_PATH).join(file);
    let mut f = File::open(pit_path).unwrap();
    let mut data: Vec<u8> = Vec::new();
    f.read_to_end(&mut data).unwrap();

    Pit::deserialize(&data).unwrap();
}