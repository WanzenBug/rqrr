/* ***********************************************************************
 * QR-code version information database
 */
#[derive(Copy, Clone)]
pub struct quirc_rs_params {
    pub bs: i32,
    pub dw: i32,
    pub ns: i32,
}
#[derive(Copy, Clone)]
pub struct quirc_version_info {
    pub data_bytes: i32,
    pub apat: [i32; 7],
    pub ecc: [quirc_rs_params; 4],
}
#[no_mangle]
pub static mut quirc_version_db: [quirc_version_info; 41] = unsafe {
    [
        quirc_version_info {
            data_bytes: 0i32,
            apat: [0; 7],
            ecc: [quirc_rs_params {
                bs: 0,
                dw: 0,
                ns: 0,
            }; 4],
        },
        quirc_version_info {
            data_bytes: 26i32,
            apat: [0i32, 0, 0, 0, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 26i32,
                    dw: 16i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 26i32,
                    dw: 19i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 26i32,
                    dw: 9i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 26i32,
                    dw: 13i32,
                    ns: 1i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 44i32,
            apat: [6i32, 18i32, 0i32, 0, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 44i32,
                    dw: 28i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 44i32,
                    dw: 34i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 44i32,
                    dw: 16i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 44i32,
                    dw: 22i32,
                    ns: 1i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 70i32,
            apat: [6i32, 22i32, 0i32, 0, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 70i32,
                    dw: 44i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 70i32,
                    dw: 55i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 35i32,
                    dw: 13i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 35i32,
                    dw: 17i32,
                    ns: 2i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 100i32,
            apat: [6i32, 26i32, 0i32, 0, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 50i32,
                    dw: 32i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 100i32,
                    dw: 80i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 25i32,
                    dw: 9i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 50i32,
                    dw: 24i32,
                    ns: 2i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 134i32,
            apat: [6i32, 30i32, 0i32, 0, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 67i32,
                    dw: 43i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 134i32,
                    dw: 108i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 33i32,
                    dw: 11i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 33i32,
                    dw: 15i32,
                    ns: 2i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 172i32,
            apat: [6i32, 34i32, 0i32, 0, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 43i32,
                    dw: 27i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 86i32,
                    dw: 68i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 43i32,
                    dw: 15i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 43i32,
                    dw: 19i32,
                    ns: 4i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 196i32,
            apat: [6i32, 22i32, 38i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 49i32,
                    dw: 31i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 98i32,
                    dw: 78i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 39i32,
                    dw: 13i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 32i32,
                    dw: 14i32,
                    ns: 2i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 242i32,
            apat: [6i32, 24i32, 42i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 60i32,
                    dw: 38i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 121i32,
                    dw: 97i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 40i32,
                    dw: 14i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 40i32,
                    dw: 18i32,
                    ns: 4i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 292i32,
            apat: [6i32, 26i32, 46i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 58i32,
                    dw: 36i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 146i32,
                    dw: 116i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 36i32,
                    dw: 12i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 36i32,
                    dw: 16i32,
                    ns: 4i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 346i32,
            apat: [6i32, 28i32, 50i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 69i32,
                    dw: 43i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 86i32,
                    dw: 68i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 43i32,
                    dw: 15i32,
                    ns: 6i32,
                },
                quirc_rs_params {
                    bs: 43i32,
                    dw: 19i32,
                    ns: 6i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 404i32,
            apat: [6i32, 30i32, 54i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 80i32,
                    dw: 50i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 101i32,
                    dw: 81i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 36i32,
                    dw: 12i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 50i32,
                    dw: 22i32,
                    ns: 4i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 466i32,
            apat: [6i32, 32i32, 58i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 58i32,
                    dw: 36i32,
                    ns: 6i32,
                },
                quirc_rs_params {
                    bs: 116i32,
                    dw: 92i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 42i32,
                    dw: 14i32,
                    ns: 7i32,
                },
                quirc_rs_params {
                    bs: 46i32,
                    dw: 20i32,
                    ns: 4i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 532i32,
            apat: [6i32, 34i32, 62i32, 0i32, 0, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 59i32,
                    dw: 37i32,
                    ns: 8i32,
                },
                quirc_rs_params {
                    bs: 133i32,
                    dw: 107i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 33i32,
                    dw: 11i32,
                    ns: 12i32,
                },
                quirc_rs_params {
                    bs: 44i32,
                    dw: 20i32,
                    ns: 8i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 581i32,
            apat: [6i32, 26i32, 46i32, 66i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 64i32,
                    dw: 40i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 145i32,
                    dw: 115i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 36i32,
                    dw: 12i32,
                    ns: 11i32,
                },
                quirc_rs_params {
                    bs: 36i32,
                    dw: 16i32,
                    ns: 11i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 655i32,
            apat: [6i32, 26i32, 48i32, 70i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 65i32,
                    dw: 41i32,
                    ns: 5i32,
                },
                quirc_rs_params {
                    bs: 109i32,
                    dw: 87i32,
                    ns: 5i32,
                },
                quirc_rs_params {
                    bs: 36i32,
                    dw: 12i32,
                    ns: 11i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 5i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 733i32,
            apat: [6i32, 26i32, 50i32, 74i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 73i32,
                    dw: 45i32,
                    ns: 7i32,
                },
                quirc_rs_params {
                    bs: 122i32,
                    dw: 98i32,
                    ns: 5i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 43i32,
                    dw: 19i32,
                    ns: 15i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 815i32,
            apat: [6i32, 30i32, 54i32, 78i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 10i32,
                },
                quirc_rs_params {
                    bs: 135i32,
                    dw: 107i32,
                    ns: 1i32,
                },
                quirc_rs_params {
                    bs: 42i32,
                    dw: 14i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 50i32,
                    dw: 22i32,
                    ns: 1i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 901i32,
            apat: [6i32, 30i32, 56i32, 82i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 69i32,
                    dw: 43i32,
                    ns: 9i32,
                },
                quirc_rs_params {
                    bs: 150i32,
                    dw: 120i32,
                    ns: 5i32,
                },
                quirc_rs_params {
                    bs: 42i32,
                    dw: 14i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 50i32,
                    dw: 22i32,
                    ns: 17i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 991i32,
            apat: [6i32, 30i32, 58i32, 86i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 70i32,
                    dw: 44i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 141i32,
                    dw: 113i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 39i32,
                    dw: 13i32,
                    ns: 9i32,
                },
                quirc_rs_params {
                    bs: 47i32,
                    dw: 21i32,
                    ns: 17i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1085i32,
            apat: [6i32, 34i32, 62i32, 90i32, 0i32, 0, 0],
            ecc: [
                quirc_rs_params {
                    bs: 67i32,
                    dw: 41i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 135i32,
                    dw: 107i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 43i32,
                    dw: 15i32,
                    ns: 15i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 15i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1156i32,
            apat: [6i32, 28i32, 50i32, 72i32, 92i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 68i32,
                    dw: 42i32,
                    ns: 17i32,
                },
                quirc_rs_params {
                    bs: 144i32,
                    dw: 116i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 46i32,
                    dw: 16i32,
                    ns: 19i32,
                },
                quirc_rs_params {
                    bs: 50i32,
                    dw: 22i32,
                    ns: 17i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1258i32,
            apat: [6i32, 26i32, 50i32, 74i32, 98i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 17i32,
                },
                quirc_rs_params {
                    bs: 139i32,
                    dw: 111i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 37i32,
                    dw: 13i32,
                    ns: 34i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 7i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1364i32,
            apat: [6i32, 30i32, 54i32, 78i32, 102i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 151i32,
                    dw: 121i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 16i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 11i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1474i32,
            apat: [6i32, 28i32, 54i32, 80i32, 106i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 73i32,
                    dw: 45i32,
                    ns: 6i32,
                },
                quirc_rs_params {
                    bs: 147i32,
                    dw: 117i32,
                    ns: 6i32,
                },
                quirc_rs_params {
                    bs: 46i32,
                    dw: 16i32,
                    ns: 30i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 11i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1588i32,
            apat: [6i32, 32i32, 58i32, 84i32, 110i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 8i32,
                },
                quirc_rs_params {
                    bs: 132i32,
                    dw: 106i32,
                    ns: 8i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 22i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 7i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1706i32,
            apat: [6i32, 30i32, 58i32, 86i32, 114i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 19i32,
                },
                quirc_rs_params {
                    bs: 142i32,
                    dw: 114i32,
                    ns: 10i32,
                },
                quirc_rs_params {
                    bs: 46i32,
                    dw: 16i32,
                    ns: 33i32,
                },
                quirc_rs_params {
                    bs: 50i32,
                    dw: 22i32,
                    ns: 28i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1828i32,
            apat: [6i32, 34i32, 62i32, 90i32, 118i32, 0i32, 0],
            ecc: [
                quirc_rs_params {
                    bs: 73i32,
                    dw: 45i32,
                    ns: 22i32,
                },
                quirc_rs_params {
                    bs: 152i32,
                    dw: 122i32,
                    ns: 8i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 12i32,
                },
                quirc_rs_params {
                    bs: 53i32,
                    dw: 23i32,
                    ns: 8i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 1921i32,
            apat: [6i32, 26i32, 50i32, 74i32, 98i32, 122i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 73i32,
                    dw: 45i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 147i32,
                    dw: 117i32,
                    ns: 3i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 11i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 4i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2051i32,
            apat: [6i32, 30i32, 54i32, 78i32, 102i32, 126i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 73i32,
                    dw: 45i32,
                    ns: 21i32,
                },
                quirc_rs_params {
                    bs: 146i32,
                    dw: 116i32,
                    ns: 7i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 19i32,
                },
                quirc_rs_params {
                    bs: 53i32,
                    dw: 23i32,
                    ns: 1i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2185i32,
            apat: [6i32, 26i32, 52i32, 78i32, 104i32, 130i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 19i32,
                },
                quirc_rs_params {
                    bs: 145i32,
                    dw: 115i32,
                    ns: 5i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 23i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 15i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2323i32,
            apat: [6i32, 30i32, 56i32, 82i32, 108i32, 134i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 145i32,
                    dw: 115i32,
                    ns: 13i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 23i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 42i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2465i32,
            apat: [6i32, 34i32, 60i32, 86i32, 112i32, 138i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 10i32,
                },
                quirc_rs_params {
                    bs: 145i32,
                    dw: 115i32,
                    ns: 17i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 19i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 10i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2611i32,
            apat: [6i32, 30i32, 58i32, 86i32, 114i32, 142i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 14i32,
                },
                quirc_rs_params {
                    bs: 145i32,
                    dw: 115i32,
                    ns: 17i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 11i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 29i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2761i32,
            apat: [6i32, 34i32, 62i32, 90i32, 118i32, 146i32, 0i32],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 14i32,
                },
                quirc_rs_params {
                    bs: 145i32,
                    dw: 115i32,
                    ns: 13i32,
                },
                quirc_rs_params {
                    bs: 46i32,
                    dw: 16i32,
                    ns: 59i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 44i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 2876i32,
            apat: [6i32, 30i32, 54i32, 78i32, 102i32, 126i32, 150i32],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 12i32,
                },
                quirc_rs_params {
                    bs: 151i32,
                    dw: 121i32,
                    ns: 12i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 22i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 39i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 3034i32,
            apat: [6i32, 24i32, 50i32, 76i32, 102i32, 128i32, 154i32],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 6i32,
                },
                quirc_rs_params {
                    bs: 151i32,
                    dw: 121i32,
                    ns: 6i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 2i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 46i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 3196i32,
            apat: [6i32, 28i32, 54i32, 80i32, 106i32, 132i32, 158i32],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 29i32,
                },
                quirc_rs_params {
                    bs: 152i32,
                    dw: 122i32,
                    ns: 17i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 24i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 49i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 3362i32,
            apat: [6i32, 32i32, 58i32, 84i32, 110i32, 136i32, 162i32],
            ecc: [
                quirc_rs_params {
                    bs: 74i32,
                    dw: 46i32,
                    ns: 13i32,
                },
                quirc_rs_params {
                    bs: 152i32,
                    dw: 122i32,
                    ns: 4i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 42i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 48i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 3532i32,
            apat: [6i32, 26i32, 54i32, 82i32, 110i32, 138i32, 166i32],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 40i32,
                },
                quirc_rs_params {
                    bs: 147i32,
                    dw: 117i32,
                    ns: 20i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 10i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 43i32,
                },
            ],
        },
        quirc_version_info {
            data_bytes: 3706i32,
            apat: [6i32, 30i32, 58i32, 86i32, 114i32, 142i32, 170i32],
            ecc: [
                quirc_rs_params {
                    bs: 75i32,
                    dw: 47i32,
                    ns: 18i32,
                },
                quirc_rs_params {
                    bs: 148i32,
                    dw: 118i32,
                    ns: 19i32,
                },
                quirc_rs_params {
                    bs: 45i32,
                    dw: 15i32,
                    ns: 20i32,
                },
                quirc_rs_params {
                    bs: 54i32,
                    dw: 24i32,
                    ns: 34i32,
                },
            ],
        },
    ]
};
