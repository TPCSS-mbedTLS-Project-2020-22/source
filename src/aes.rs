#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused)]

use std::convert::{TryFrom, TryInto};
// Original C comment: padlock.c and aesni.c rely on these values!
pub const MBEDTLS_AES_ENCRYPT: isize = 1;
pub const MBEDTLS_AES_DECRYPT: isize = 0;

// Error codes in range 0x0020-0x0022
pub const MBEDTLS_ERR_AES_INVALID_KEY_LENGTH: isize = -0x0020; // Invalid key length
pub const MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH: isize = -0x0022; // Invalid data input length.

// Error codes in range 0x0021-0x0025
pub const MBEDTLS_ERR_AES_BAD_INPUT_DATA: isize = -0x0021; // Invalid input data.

// MBEDTLS_ERR_AES_FEATURE_UNAVAILABLE is deprecated and should not be used.
pub const MBEDTLS_ERR_AES_FEATURE_UNAVAILABLE: isize = -0x0023; // Feature not available. For example, an unsupported AES key size.

// MBEDTLS_ERR_AES_HW_ACCEL_FAILED is deprecated and should not be used.
pub const MBEDTLS_ERR_AES_HW_ACCEL_FAILED: isize = -0x0025; // AES hardware accelerator failed.

/*
 * Forward S-box
 */
const FSBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];


/*
 * Reverse S-Box
 */
const RSBOX: [u8; 256] = [
    /* 0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F  */
       0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
       0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
       0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
       0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
       0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
       0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
       0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
       0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
       0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
       0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
       0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
       0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
       0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
       0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef,
       0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
       0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d
   ];


/* 
 * GF Multiplication by 2
 */
const MUL2: [u8; 256] = [
    0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1a, 0x1c, 0x1e,
    0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e, 0x30, 0x32, 0x34, 0x36, 0x38, 0x3a, 0x3c, 0x3e,
    0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x50, 0x52, 0x54, 0x56, 0x58, 0x5a, 0x5c, 0x5e,
    0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0x6c, 0x6e, 0x70, 0x72, 0x74, 0x76, 0x78, 0x7a, 0x7c, 0x7e,
    0x80, 0x82, 0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8e, 0x90, 0x92, 0x94, 0x96, 0x98, 0x9a, 0x9c, 0x9e,
    0xa0, 0xa2, 0xa4, 0xa6, 0xa8, 0xaa, 0xac, 0xae, 0xb0, 0xb2, 0xb4, 0xb6, 0xb8, 0xba, 0xbc, 0xbe,
    0xc0, 0xc2, 0xc4, 0xc6, 0xc8, 0xca, 0xcc, 0xce, 0xd0, 0xd2, 0xd4, 0xd6, 0xd8, 0xda, 0xdc, 0xde,
    0xe0, 0xe2, 0xe4, 0xe6, 0xe8, 0xea, 0xec, 0xee, 0xf0, 0xf2, 0xf4, 0xf6, 0xf8, 0xfa, 0xfc, 0xfe,
    0x1b, 0x19, 0x1f, 0x1d, 0x13, 0x11, 0x17, 0x15, 0x0b, 0x09, 0x0f, 0x0d, 0x03, 0x01, 0x07, 0x05,
    0x3b, 0x39, 0x3f, 0x3d, 0x33, 0x31, 0x37, 0x35, 0x2b, 0x29, 0x2f, 0x2d, 0x23, 0x21, 0x27, 0x25,
    0x5b, 0x59, 0x5f, 0x5d, 0x53, 0x51, 0x57, 0x55, 0x4b, 0x49, 0x4f, 0x4d, 0x43, 0x41, 0x47, 0x45,
    0x7b, 0x79, 0x7f, 0x7d, 0x73, 0x71, 0x77, 0x75, 0x6b, 0x69, 0x6f, 0x6d, 0x63, 0x61, 0x67, 0x65,
    0x9b, 0x99, 0x9f, 0x9d, 0x93, 0x91, 0x97, 0x95, 0x8b, 0x89, 0x8f, 0x8d, 0x83, 0x81, 0x87, 0x85,
    0xbb, 0xb9, 0xbf, 0xbd, 0xb3, 0xb1, 0xb7, 0xb5, 0xab, 0xa9, 0xaf, 0xad, 0xa3, 0xa1, 0xa7, 0xa5,
    0xdb, 0xd9, 0xdf, 0xdd, 0xd3, 0xd1, 0xd7, 0xd5, 0xcb, 0xc9, 0xcf, 0xcd, 0xc3, 0xc1, 0xc7, 0xc5,
    0xfb, 0xf9, 0xff, 0xfd, 0xf3, 0xf1, 0xf7, 0xf5, 0xeb, 0xe9, 0xef, 0xed, 0xe3, 0xe1, 0xe7, 0xe5,
];


/* 
 * GF Multiplication by 3
 */
const MUL3: [u8; 256] = [
    0x00, 0x03, 0x06, 0x05, 0x0c, 0x0f, 0x0a, 0x09, 0x18, 0x1b, 0x1e, 0x1d, 0x14, 0x17, 0x12, 0x11,
    0x30, 0x33, 0x36, 0x35, 0x3c, 0x3f, 0x3a, 0x39, 0x28, 0x2b, 0x2e, 0x2d, 0x24, 0x27, 0x22, 0x21,
    0x60, 0x63, 0x66, 0x65, 0x6c, 0x6f, 0x6a, 0x69, 0x78, 0x7b, 0x7e, 0x7d, 0x74, 0x77, 0x72, 0x71,
    0x50, 0x53, 0x56, 0x55, 0x5c, 0x5f, 0x5a, 0x59, 0x48, 0x4b, 0x4e, 0x4d, 0x44, 0x47, 0x42, 0x41,
    0xc0, 0xc3, 0xc6, 0xc5, 0xcc, 0xcf, 0xca, 0xc9, 0xd8, 0xdb, 0xde, 0xdd, 0xd4, 0xd7, 0xd2, 0xd1,
    0xf0, 0xf3, 0xf6, 0xf5, 0xfc, 0xff, 0xfa, 0xf9, 0xe8, 0xeb, 0xee, 0xed, 0xe4, 0xe7, 0xe2, 0xe1,
    0xa0, 0xa3, 0xa6, 0xa5, 0xac, 0xaf, 0xaa, 0xa9, 0xb8, 0xbb, 0xbe, 0xbd, 0xb4, 0xb7, 0xb2, 0xb1,
    0x90, 0x93, 0x96, 0x95, 0x9c, 0x9f, 0x9a, 0x99, 0x88, 0x8b, 0x8e, 0x8d, 0x84, 0x87, 0x82, 0x81,
    0x9b, 0x98, 0x9d, 0x9e, 0x97, 0x94, 0x91, 0x92, 0x83, 0x80, 0x85, 0x86, 0x8f, 0x8c, 0x89, 0x8a,
    0xab, 0xa8, 0xad, 0xae, 0xa7, 0xa4, 0xa1, 0xa2, 0xb3, 0xb0, 0xb5, 0xb6, 0xbf, 0xbc, 0xb9, 0xba,
    0xfb, 0xf8, 0xfd, 0xfe, 0xf7, 0xf4, 0xf1, 0xf2, 0xe3, 0xe0, 0xe5, 0xe6, 0xef, 0xec, 0xe9, 0xea,
    0xcb, 0xc8, 0xcd, 0xce, 0xc7, 0xc4, 0xc1, 0xc2, 0xd3, 0xd0, 0xd5, 0xd6, 0xdf, 0xdc, 0xd9, 0xda,
    0x5b, 0x58, 0x5d, 0x5e, 0x57, 0x54, 0x51, 0x52, 0x43, 0x40, 0x45, 0x46, 0x4f, 0x4c, 0x49, 0x4a,
    0x6b, 0x68, 0x6d, 0x6e, 0x67, 0x64, 0x61, 0x62, 0x73, 0x70, 0x75, 0x76, 0x7f, 0x7c, 0x79, 0x7a,
    0x3b, 0x38, 0x3d, 0x3e, 0x37, 0x34, 0x31, 0x32, 0x23, 0x20, 0x25, 0x26, 0x2f, 0x2c, 0x29, 0x2a,
    0x0b, 0x08, 0x0d, 0x0e, 0x07, 0x04, 0x01, 0x02, 0x13, 0x10, 0x15, 0x16, 0x1f, 0x1c, 0x19, 0x1a,
];


/* 
 * GF Multiplication by 9
 */
const MUL9: [u8; 256] = [
    0x00,0x09,0x12,0x1b,0x24,0x2d,0x36,0x3f,0x48,0x41,0x5a,0x53,0x6c,0x65,0x7e,0x77,
    0x90,0x99,0x82,0x8b,0xb4,0xbd,0xa6,0xaf,0xd8,0xd1,0xca,0xc3,0xfc,0xf5,0xee,0xe7,
    0x3b,0x32,0x29,0x20,0x1f,0x16,0x0d,0x04,0x73,0x7a,0x61,0x68,0x57,0x5e,0x45,0x4c,
    0xab,0xa2,0xb9,0xb0,0x8f,0x86,0x9d,0x94,0xe3,0xea,0xf1,0xf8,0xc7,0xce,0xd5,0xdc,
    0x76,0x7f,0x64,0x6d,0x52,0x5b,0x40,0x49,0x3e,0x37,0x2c,0x25,0x1a,0x13,0x08,0x01,
    0xe6,0xef,0xf4,0xfd,0xc2,0xcb,0xd0,0xd9,0xae,0xa7,0xbc,0xb5,0x8a,0x83,0x98,0x91,
    0x4d,0x44,0x5f,0x56,0x69,0x60,0x7b,0x72,0x05,0x0c,0x17,0x1e,0x21,0x28,0x33,0x3a,
    0xdd,0xd4,0xcf,0xc6,0xf9,0xf0,0xeb,0xe2,0x95,0x9c,0x87,0x8e,0xb1,0xb8,0xa3,0xaa,
    0xec,0xe5,0xfe,0xf7,0xc8,0xc1,0xda,0xd3,0xa4,0xad,0xb6,0xbf,0x80,0x89,0x92,0x9b,
    0x7c,0x75,0x6e,0x67,0x58,0x51,0x4a,0x43,0x34,0x3d,0x26,0x2f,0x10,0x19,0x02,0x0b,
    0xd7,0xde,0xc5,0xcc,0xf3,0xfa,0xe1,0xe8,0x9f,0x96,0x8d,0x84,0xbb,0xb2,0xa9,0xa0,
    0x47,0x4e,0x55,0x5c,0x63,0x6a,0x71,0x78,0x0f,0x06,0x1d,0x14,0x2b,0x22,0x39,0x30,
    0x9a,0x93,0x88,0x81,0xbe,0xb7,0xac,0xa5,0xd2,0xdb,0xc0,0xc9,0xf6,0xff,0xe4,0xed,
    0x0a,0x03,0x18,0x11,0x2e,0x27,0x3c,0x35,0x42,0x4b,0x50,0x59,0x66,0x6f,0x74,0x7d,
    0xa1,0xa8,0xb3,0xba,0x85,0x8c,0x97,0x9e,0xe9,0xe0,0xfb,0xf2,0xcd,0xc4,0xdf,0xd6,
    0x31,0x38,0x23,0x2a,0x15,0x1c,0x07,0x0e,0x79,0x70,0x6b,0x62,0x5d,0x54,0x4f,0x46
];


/* 
 * GF Multiplication by 11
 */
const MUL11: [u8; 256] = [
    0x00,0x0b,0x16,0x1d,0x2c,0x27,0x3a,0x31,0x58,0x53,0x4e,0x45,0x74,0x7f,0x62,0x69,
    0xb0,0xbb,0xa6,0xad,0x9c,0x97,0x8a,0x81,0xe8,0xe3,0xfe,0xf5,0xc4,0xcf,0xd2,0xd9,
    0x7b,0x70,0x6d,0x66,0x57,0x5c,0x41,0x4a,0x23,0x28,0x35,0x3e,0x0f,0x04,0x19,0x12,
    0xcb,0xc0,0xdd,0xd6,0xe7,0xec,0xf1,0xfa,0x93,0x98,0x85,0x8e,0xbf,0xb4,0xa9,0xa2,
    0xf6,0xfd,0xe0,0xeb,0xda,0xd1,0xcc,0xc7,0xae,0xa5,0xb8,0xb3,0x82,0x89,0x94,0x9f,
    0x46,0x4d,0x50,0x5b,0x6a,0x61,0x7c,0x77,0x1e,0x15,0x08,0x03,0x32,0x39,0x24,0x2f,
    0x8d,0x86,0x9b,0x90,0xa1,0xaa,0xb7,0xbc,0xd5,0xde,0xc3,0xc8,0xf9,0xf2,0xef,0xe4,
    0x3d,0x36,0x2b,0x20,0x11,0x1a,0x07,0x0c,0x65,0x6e,0x73,0x78,0x49,0x42,0x5f,0x54,
    0xf7,0xfc,0xe1,0xea,0xdb,0xd0,0xcd,0xc6,0xaf,0xa4,0xb9,0xb2,0x83,0x88,0x95,0x9e,
    0x47,0x4c,0x51,0x5a,0x6b,0x60,0x7d,0x76,0x1f,0x14,0x09,0x02,0x33,0x38,0x25,0x2e,
    0x8c,0x87,0x9a,0x91,0xa0,0xab,0xb6,0xbd,0xd4,0xdf,0xc2,0xc9,0xf8,0xf3,0xee,0xe5,
    0x3c,0x37,0x2a,0x21,0x10,0x1b,0x06,0x0d,0x64,0x6f,0x72,0x79,0x48,0x43,0x5e,0x55,
    0x01,0x0a,0x17,0x1c,0x2d,0x26,0x3b,0x30,0x59,0x52,0x4f,0x44,0x75,0x7e,0x63,0x68,
    0xb1,0xba,0xa7,0xac,0x9d,0x96,0x8b,0x80,0xe9,0xe2,0xff,0xf4,0xc5,0xce,0xd3,0xd8,
    0x7a,0x71,0x6c,0x67,0x56,0x5d,0x40,0x4b,0x22,0x29,0x34,0x3f,0x0e,0x05,0x18,0x13,
    0xca,0xc1,0xdc,0xd7,0xe6,0xed,0xf0,0xfb,0x92,0x99,0x84,0x8f,0xbe,0xb5,0xa8,0xa3
];


/* 
 * GF Multiplication by 13
 */
const MUL13: [u8; 256] = [
    0x00,0x0d,0x1a,0x17,0x34,0x39,0x2e,0x23,0x68,0x65,0x72,0x7f,0x5c,0x51,0x46,0x4b,
    0xd0,0xdd,0xca,0xc7,0xe4,0xe9,0xfe,0xf3,0xb8,0xb5,0xa2,0xaf,0x8c,0x81,0x96,0x9b,
    0xbb,0xb6,0xa1,0xac,0x8f,0x82,0x95,0x98,0xd3,0xde,0xc9,0xc4,0xe7,0xea,0xfd,0xf0,
    0x6b,0x66,0x71,0x7c,0x5f,0x52,0x45,0x48,0x03,0x0e,0x19,0x14,0x37,0x3a,0x2d,0x20,
    0x6d,0x60,0x77,0x7a,0x59,0x54,0x43,0x4e,0x05,0x08,0x1f,0x12,0x31,0x3c,0x2b,0x26,
    0xbd,0xb0,0xa7,0xaa,0x89,0x84,0x93,0x9e,0xd5,0xd8,0xcf,0xc2,0xe1,0xec,0xfb,0xf6,
    0xd6,0xdb,0xcc,0xc1,0xe2,0xef,0xf8,0xf5,0xbe,0xb3,0xa4,0xa9,0x8a,0x87,0x90,0x9d,
    0x06,0x0b,0x1c,0x11,0x32,0x3f,0x28,0x25,0x6e,0x63,0x74,0x79,0x5a,0x57,0x40,0x4d,
    0xda,0xd7,0xc0,0xcd,0xee,0xe3,0xf4,0xf9,0xb2,0xbf,0xa8,0xa5,0x86,0x8b,0x9c,0x91,
    0x0a,0x07,0x10,0x1d,0x3e,0x33,0x24,0x29,0x62,0x6f,0x78,0x75,0x56,0x5b,0x4c,0x41,
    0x61,0x6c,0x7b,0x76,0x55,0x58,0x4f,0x42,0x09,0x04,0x13,0x1e,0x3d,0x30,0x27,0x2a,
    0xb1,0xbc,0xab,0xa6,0x85,0x88,0x9f,0x92,0xd9,0xd4,0xc3,0xce,0xed,0xe0,0xf7,0xfa,
    0xb7,0xba,0xad,0xa0,0x83,0x8e,0x99,0x94,0xdf,0xd2,0xc5,0xc8,0xeb,0xe6,0xf1,0xfc,
    0x67,0x6a,0x7d,0x70,0x53,0x5e,0x49,0x44,0x0f,0x02,0x15,0x18,0x3b,0x36,0x21,0x2c,
    0x0c,0x01,0x16,0x1b,0x38,0x35,0x22,0x2f,0x64,0x69,0x7e,0x73,0x50,0x5d,0x4a,0x47,
    0xdc,0xd1,0xc6,0xcb,0xe8,0xe5,0xf2,0xff,0xb4,0xb9,0xae,0xa3,0x80,0x8d,0x9a,0x97
];


/* 
 * GF Multiplication by 14
 */
const MUL14: [u8; 256] = [
    0x00,0x0e,0x1c,0x12,0x38,0x36,0x24,0x2a,0x70,0x7e,0x6c,0x62,0x48,0x46,0x54,0x5a,
    0xe0,0xee,0xfc,0xf2,0xd8,0xd6,0xc4,0xca,0x90,0x9e,0x8c,0x82,0xa8,0xa6,0xb4,0xba,
    0xdb,0xd5,0xc7,0xc9,0xe3,0xed,0xff,0xf1,0xab,0xa5,0xb7,0xb9,0x93,0x9d,0x8f,0x81,
    0x3b,0x35,0x27,0x29,0x03,0x0d,0x1f,0x11,0x4b,0x45,0x57,0x59,0x73,0x7d,0x6f,0x61,
    0xad,0xa3,0xb1,0xbf,0x95,0x9b,0x89,0x87,0xdd,0xd3,0xc1,0xcf,0xe5,0xeb,0xf9,0xf7,
    0x4d,0x43,0x51,0x5f,0x75,0x7b,0x69,0x67,0x3d,0x33,0x21,0x2f,0x05,0x0b,0x19,0x17,
    0x76,0x78,0x6a,0x64,0x4e,0x40,0x52,0x5c,0x06,0x08,0x1a,0x14,0x3e,0x30,0x22,0x2c,
    0x96,0x98,0x8a,0x84,0xae,0xa0,0xb2,0xbc,0xe6,0xe8,0xfa,0xf4,0xde,0xd0,0xc2,0xcc,
    0x41,0x4f,0x5d,0x53,0x79,0x77,0x65,0x6b,0x31,0x3f,0x2d,0x23,0x09,0x07,0x15,0x1b,
    0xa1,0xaf,0xbd,0xb3,0x99,0x97,0x85,0x8b,0xd1,0xdf,0xcd,0xc3,0xe9,0xe7,0xf5,0xfb,
    0x9a,0x94,0x86,0x88,0xa2,0xac,0xbe,0xb0,0xea,0xe4,0xf6,0xf8,0xd2,0xdc,0xce,0xc0,
    0x7a,0x74,0x66,0x68,0x42,0x4c,0x5e,0x50,0x0a,0x04,0x16,0x18,0x32,0x3c,0x2e,0x20,
    0xec,0xe2,0xf0,0xfe,0xd4,0xda,0xc8,0xc6,0x9c,0x92,0x80,0x8e,0xa4,0xaa,0xb8,0xb6,
    0x0c,0x02,0x10,0x1e,0x34,0x3a,0x28,0x26,0x7c,0x72,0x60,0x6e,0x44,0x4a,0x58,0x56,
    0x37,0x39,0x2b,0x25,0x0f,0x01,0x13,0x1d,0x47,0x49,0x5b,0x55,0x7f,0x71,0x63,0x6d,
    0xd7,0xd9,0xcb,0xc5,0xef,0xe1,0xf3,0xfd,0xa7,0xa9,0xbb,0xb5,0x9f,0x91,0x83,0x8d
];


/* 
 * Rcon values
 */
const RCON: [u8; 256] = [
    0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a,
    0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39,
    0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a,
    0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8,
    0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef,
    0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc,
    0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b,
    0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3,
    0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94,
    0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20,
    0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35,
    0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f,
    0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04,
    0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63,
    0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd,
    0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d,
];

pub fn GET_UINT32_LE(n: &mut u32, b: &[u8], i: usize) {
    *n = (b[i] as u32) << 24;
    *n = ((b[i + 1] as u32) << 16) | *n;
    *n = ((b[i + 2] as u32) << 8) | *n;
    *n = (b[i + 3] as u32)  | *n;
}

pub fn PUT_UINT32_LE(n: u32, b: &mut [u8], i: usize) {
    b[i] = (n >> 24) as u8;
    b[i + 1] = (n >> 16) as u8;
    b[i + 2] = (n >> 8) as u8;
    b[i + 3] = n as u8;
}

/*
 * \brief The AES context-type definition.
 */
#[derive(Debug)]
pub struct mbedtls_aes_context {
    pub nr: usize, /*< The number of rounds. */
    pub rk: [u32; 44],   //< AES round keys.
    pub buf: [u32; 68], /*< Unaligned data buffer. This buffer can
               hold 32 extra Bytes, which can be used for
               one of the following purposes:
               <ul><li>Alignment if VIA padlock is
                       used.</li>
               <li>Simplifying key expansion in the 256-bit
                   case by generating an extra round key.
                   </li></ul> */
    pub key: [u32; 4],
    pub key8: [u8; 16],
    pub expanded_key8: [u8; 176]
}

fn sub_bytes(state: &mut [u8; 16]) {
    for i in 0..16 {
        state[i] = FSBOX[state[i] as usize];
    }
}

fn shift_rows(state: &mut [u8; 16]) {
    let mut tmp: [u8; 16] = [0; 16];
    tmp[0] = state[0];
    tmp[1] = state[5];
    tmp[2] = state[10];
    tmp[3] = state[15];

    tmp[4] = state[4];
    tmp[5] = state[9];
    tmp[6] = state[14];
    tmp[7] = state[3];

    tmp[8] = state[8];
    tmp[9] = state[13];
    tmp[10] = state[2];
    tmp[11] = state[7];

    tmp[12] = state[12];
    tmp[13] = state[1];
    tmp[14] = state[6];
    tmp[15] = state[11];

    for i in 0..16 {
        state[i] = tmp[i];
    }
}

fn mix_columns(state: &mut [u8; 16]) {
    for i in 0..4 {
        mix_single_column(&mut state[(i * 4)..(i * 4 + 4)]);
    }
}

pub fn mix_single_column(state: &mut [u8]) {
    let mut tmp: [u8; 4] = [0; 4];
    tmp[0] = MUL2[state[0] as usize] ^ MUL3[state[1] as usize] ^ state[2] ^ state[3];
    tmp[1] = state[0] ^ MUL2[state[1] as usize] ^ MUL3[state[2] as usize] ^ state[3];
    tmp[2] = state[0] ^ state[1] ^ MUL2[state[2] as usize] ^ MUL3[state[3] as usize];
    tmp[3] = MUL3[state[0] as usize] ^ state[1] ^ state[2] ^ MUL2[state[3] as usize];

    for i in 0..4 {
        state[i] = tmp[i];
    }
}

fn add_round_key(state: &mut [u8; 16], round_key: &[u8; 16]) {
    for i in 0..16 {
        state[i] ^= round_key[i];
    }
}

fn rot_word(word: [u8; 4]) -> [u8; 4]
{
    // let t: [u8; 4] = [word[1], word[2], word[3], word[0]];
    // t
    [word[1], word[2], word[3], word[0]]
}


fn sub_word(word: [u8; 4]) -> [u8; 4]
{
    // word[0] = FSBOX[word[0] as usize];
    // word[1] = FSBOX[word[1] as usize];
    // word[2] = FSBOX[word[2] as usize];
    // word[3] = FSBOX[word[3] as usize];

    // word
    [FSBOX[word[0] as usize], FSBOX[word[1] as usize], FSBOX[word[2] as usize], FSBOX[word[3] as usize]]
}

impl mbedtls_aes_context{
    pub fn new() -> mbedtls_aes_context
    {
        mbedtls_aes_context{nr: 10, rk: [0u32; 44], buf: [0u32; 68], key: [0u32; 4], key8: [0; 16], expanded_key8: [0; 176]}
    }

    // Function to expand keys where keys are stored in u32 values
    pub fn key_expansion32(&mut self, input32: &[u32; 4]) //, expanded_key32: &mut [u32; 44])
    {
        let mut input8: [u8; 16] = [0; 16];
        // let mut expanded_key8: [u8; 176] = [0; 176];
        // self.key8 = [0; 16];
        // self.expanded_key8 = [0; 176];

        // convert u32 key to u8 key
        for i in 0..4
        {
            PUT_UINT32_LE(input32[i], &mut input8, 4 * i);
        }

        // Expand it into u8 key
        // mbedtls_aes_context::key_expansion8(&self.key8, &mut self.expanded_key8);
        self.key_expansion8(&input8);

        // Change it back to u32 expanded key
        // for i in (0..176).step_by(4)
        // {
        //     GET_UINT32_LE(&mut self.rk[i / 4], &self.expanded_key8, i);
        // }

        // move key8 and expanded_key

    }

    // Function to expand keys where keys are stored in u8 values
    pub fn key_expansion8(&mut self, input_key: &[u8; 16]){ //, expanded_key: &mut [u8; 176]){
        for i in 0..16
        {
            self.key8[i] = input_key[i];
        }

        const N: usize = 4;
        const R: usize = 11;
        for i in 0..(4 * R)
        {
            if i < N
            {
                self.expanded_key8[4 * i + 0] = input_key[4 * i];
                self.expanded_key8[4 * i + 1] = input_key[4 * i + 1];
                self.expanded_key8[4 * i + 2] = input_key[4 * i + 2];
                self.expanded_key8[4 * i + 3] = input_key[4 * i + 3];
            }
            else if i >= N && i % N == 0
            {
                let sub_rot: [u8; 4] = sub_word(rot_word(self.expanded_key8[4 * (i - 1)..4 * i].try_into().expect("Error")));
                self.expanded_key8[4 * i + 0] = self.expanded_key8[4 * (i - N)] ^ sub_rot[0] ^ RCON[i / N];
                self.expanded_key8[4 * i + 1] = self.expanded_key8[4 * (i - N) + 1] ^ sub_rot[1];
                self.expanded_key8[4 * i + 2] = self.expanded_key8[4 * (i - N) + 2] ^ sub_rot[2];
                self.expanded_key8[4 * i + 3] = self.expanded_key8[4 * (i - N) + 3] ^ sub_rot[3];
            }
            else
            {
                self.expanded_key8[4 * i + 0] = self.expanded_key8[4 * (i - N) + 0] ^ self.expanded_key8[4 * (i - 1) + 0];
                self.expanded_key8[4 * i + 1] = self.expanded_key8[4 * (i - N) + 1] ^ self.expanded_key8[4 * (i - 1) + 1];
                self.expanded_key8[4 * i + 2] = self.expanded_key8[4 * (i - N) + 2] ^ self.expanded_key8[4 * (i - 1) + 2];
                self.expanded_key8[4 * i + 3] = self.expanded_key8[4 * (i - N) + 3] ^ self.expanded_key8[4 * (i - 1) + 3];
            }
        }

        // Put this in 32 bit key
        for i in (0..16).step_by(4)
        {
            GET_UINT32_LE(&mut self.key[i / 4], &self.key8, i);
        }

        // Put in 32 bit expanded key
        for i in (0..176).step_by(4)
        {
            GET_UINT32_LE(&mut self.rk[i / 4], &self.expanded_key8, i);
        }
    }
}

/*

/**
 * \brief The AES XTS context-type definition.
 */
struct mbedtls_aes_xts_context {
    crypt: mbedtls_aes_context, /*< The AES context to use for AES block
                                encryption or decryption. */
    tweak: mbedtls_aes_context, /*< The AES context used for tweak
                                computation. */
}

/**
 * \brief          This function initializes the specified AES context.
 *
 *                 It must be the first API called before using
 *                 the context.
 *
 * \param ctx      The AES context to initialize. This must not be \c NULL.
 */
fn mbedtls_aes_init(ctx: &mut mbedtls_aes_context) {}

/**
 * \brief          This function releases and clears the specified AES context.
 *
 * \param ctx      The AES context to clear.
 *                 If this is \c NULL, this function does nothing.
 *                 Otherwise, the context must have been at least initialized.
 */
fn mbedtls_aes_free(ctx: &mut mbedtls_aes_context) {}

/**
 * \brief          This function initializes the specified AES XTS context.
 *
 *                 It must be the first API called before using
 *                 the context.
 *
 * \param ctx      The AES XTS context to initialize. This must not be \c NULL.
 */
fn mbedtls_aes_xts_init(ctx: &mut mbedtls_aes_xts_context) {}

/**
 * \brief          This function releases and clears the specified AES XTS context.
 *
 * \param ctx      The AES XTS context to clear.
 *                 If this is \c NULL, this function does nothing.
 *                 Otherwise, the context must have been at least initialized.
 */
fn mbedtls_aes_xts_free(ctx: &mut mbedtls_aes_xts_context) {}
*/


/**
 * \brief          This function sets the encryption key.
 *
 * \param ctx      The AES context to which the key should be bound.
 *                 It must be initialized.
 * \param key      The encryption key.
 *                 This must be a readable buffer of size \p keybits bits.
 * \param keybits  The size of data passed in bits. Valid options are:
 *                 <ul><li>128 bits</li>
 *                 <li>192 bits</li>
 *                 <li>256 bits</li></ul>
 *
 * \return         \c 0 on success.
 * \return         #MBEDTLS_ERR_AES_INVALID_KEY_LENGTH on failure.
 */
pub fn mbedtls_aes_setkey_enc(ctx: &mut mbedtls_aes_context, key: &[u8; 16], keybits: usize) -> isize {
    if keybits == 128
    {
        ctx.nr = 10;
    }
    else
    {
        return MBEDTLS_ERR_AES_INVALID_KEY_LENGTH;
    }

    for i in 0..4
    {
        // ctx.key[i] = (key[4 * i] as u32) << 24;
        // ctx.key[i] = ((key[4 * i + 1] as u32) << 16) | ctx.key[i];
        // ctx.key[i] = ((key[4 * i + 2] as u32) << 8) | ctx.key[i];
        // ctx.key[i] = (key[4 * i + 3] as u32) | ctx.key[i];

        GET_UINT32_LE(&mut ctx.key[i], &key[4 * i..4 * i+4], 0);
    }
    // mbedtls_aes_context::expand_key(&ctx.key, &mut ctx.rk);
    ctx.key_expansion8(key);
    0
}



/**
 * \brief          This function sets the decryption key.
 *
 * \param ctx      The AES context to which the key should be bound.
 *                 It must be initialized.
 * \param key      The decryption key.
 *                 This must be a readable buffer of size \p keybits bits.
 * \param keybits  The size of data passed. Valid options are:
 *                 <ul><li>128 bits</li>
 *                 <li>192 bits</li>
 *                 <li>256 bits</li></ul>
 *
 * \return         \c 0 on success.
 * \return         #MBEDTLS_ERR_AES_INVALID_KEY_LENGTH on failure.
 */
pub fn mbedtls_aes_setkey_dec(ctx: &mut mbedtls_aes_context, key: &[u8; 16], keybits: usize) -> isize {
    mbedtls_aes_setkey_enc(ctx, &key, keybits)
}

/*
/**
 * \brief          This function prepares an XTS context for encryption and
 *                 sets the encryption key.
 *
 * \param ctx      The AES XTS context to which the key should be bound.
 *                 It must be initialized.
 * \param key      The encryption key. This is comprised of the XTS key1
 *                 concatenated with the XTS key2.
 *                 This must be a readable buffer of size \p keybits bits.
 * \param keybits  The size of \p key passed in bits. Valid options are:
 *                 <ul><li>256 bits (each of key1 and key2 is a 128-bit key)</li>
 *                 <li>512 bits (each of key1 and key2 is a 256-bit key)</li></ul>
 *
 * \return         \c 0 on success.
 * \return         #MBEDTLS_ERR_AES_INVALID_KEY_LENGTH on failure.
 */
fn mbedtls_aes_xts_setkey_enc(
    ctx: &mut mbedtls_aes_xts_context,
    key: &char,
    keybits: usize,
) -> isize {
    1
}

/**
 * \brief          This function prepares an XTS context for decryption and
 *                 sets the decryption key.
 *
 * \param ctx      The AES XTS context to which the key should be bound.
 *                 It must be initialized.
 * \param key      The decryption key. This is comprised of the XTS key1
 *                 concatenated with the XTS key2.
 *                 This must be a readable buffer of size \p keybits bits.
 * \param keybits  The size of \p key passed in bits. Valid options are:
 *                 <ul><li>256 bits (each of key1 and key2 is a 128-bit key)</li>
 *                 <li>512 bits (each of key1 and key2 is a 256-bit key)</li></ul>
 *
 * \return         \c 0 on success.
 * \return         #MBEDTLS_ERR_AES_INVALID_KEY_LENGTH on failure.
 */
fn mbedtls_aes_xts_setkey_dec(
    ctx: &mut mbedtls_aes_xts_context,
    key: &char,
    keybits: usize,
) -> isize {
    1
}
*/



/* *
 * \brief          This function performs an AES single-block encryption or
 *                 decryption operation.
 *
 *                 It performs the operation defined in the \p mode parameter
 *                 (encrypt or decrypt), on the input data buffer defined in
 *                 the \p input parameter.
 *
 *                 mbedtls_aes_init(), and either mbedtls_aes_setkey_enc() or
 *                 mbedtls_aes_setkey_dec() must be called before the first
 *                 call to this API with the same context.
 *
 * \param ctx      The AES context to use for encryption or decryption.
 *                 It must be initialized and bound to a key.
 * \param mode     The AES operation: #MBEDTLS_AES_ENCRYPT or
 *                 #MBEDTLS_AES_DECRYPT.
 * \param input    The buffer holding the input data.
 *                 It must be readable and at least \c 16 Bytes long.
 * \param output   The buffer where the output data will be written.
 *                 It must be writeable and at least \c 16 Bytes long.
 * \return         \c 0 on success.
 */
pub fn mbedtls_aes_crypt_ecb(
    ctx: &mbedtls_aes_context,
    mode: isize,
    input: &[u8],
    mut output: &mut [u8],
) -> isize {
    let mut temp_input: [u8; 16] = [0; 16];
    let mut temp_output: [u8; 16] = [0; 16];
    let length = input.len();

    if length % 16 != 0
    {
        return MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH;
    }

    for i in (0..length).step_by(16)
    {
        for j in 0..16
        {
            temp_input[j] = input[i + j];
        }

        if mode == MBEDTLS_AES_ENCRYPT
        {
            mbedtls_internal_aes_encrypt(&ctx, &temp_input, &mut temp_output);
        }
        else
        {
            mbedtls_internal_aes_decrypt(&ctx, &temp_input, &mut temp_output);
        }

        for j in 0..16
        {
            output[i + j] = temp_output[j];
        }
    }

    
    0
}

/* *
 * \brief           Internal AES block encryption function. This is only
 *                  exposed to allow overriding it using
 *                  \c MBEDTLS_AES_ENCRYPT_ALT.
 *
 * \param ctx       The AES context to use for encryption.
 * \param input     The plaintext block.
 * \param output    The output (ciphertext) block.
 *
 * \return          \c 0 on success.
 */
pub fn mbedtls_internal_aes_encrypt(
    ctx: &mbedtls_aes_context,
    input: &[u8; 16],
    mut output: &mut [u8; 16],
) -> isize {
    // let mut output: [u8; 16] = [0; 16];
    let mut round_key: [u8; 16] = [0; 16];
    // make a copy of the message
    for i in 0..16
    {
        output[i] = input[i];
        round_key[i] = ctx.key8[i];
    }

    add_round_key(&mut output, &round_key);

    for i in 0..(ctx.nr-1)
    {
        sub_bytes(&mut output);
        shift_rows(&mut output);
        mix_columns(&mut output);

        for j in 0..16
        {
            round_key[j] = ctx.expanded_key8[16 * (i + 1) + j];
        }

        add_round_key(&mut output, &round_key);
    }

    // Final round
    sub_bytes(&mut output);
    shift_rows(&mut output);
    for j in 0..16
    {
        round_key[j] = ctx.expanded_key8[160 + j];
    }
    add_round_key(&mut output, &round_key);
    0
}

fn inv_shift_rows(state: &mut [u8; 16]) {
    let mut tmp: [u8; 16] = [0; 16];
    tmp[0] = state[0];
    tmp[1] = state[13];
    tmp[2] = state[10];
    tmp[3] = state[7];

    tmp[4] = state[4];
    tmp[5] = state[1];
    tmp[6] = state[14];
    tmp[7] = state[11];

    tmp[8] = state[8];
    tmp[9] = state[5];
    tmp[10] = state[2];
    tmp[11] = state[15];

    tmp[12] = state[12];
    tmp[13] = state[9];
    tmp[14] = state[6];
    tmp[15] = state[3];

    for i in 0..16 {
        state[i] = tmp[i];
    }
}

fn inv_sub_bytes(state: &mut [u8; 16]) {
    for i in 0..16 {
        state[i] = RSBOX[state[i] as usize];
    }
}

fn inv_mix_columns(state: &mut [u8; 16]) {
    for i in 0..4 {
        inv_mix_single_column(&mut state[(i * 4)..(i * 4 + 4)]);
    }
}

pub fn inv_mix_single_column(state: &mut [u8]) {
    let mut tmp: [u8; 4] = [0; 4];
    tmp[0] = MUL14[state[0] as usize] ^ MUL9[state[3] as usize] ^ MUL13[state[2] as usize] ^ MUL11[state[1] as usize];
    tmp[1] = MUL14[state[1] as usize] ^ MUL9[state[0] as usize] ^ MUL13[state[3] as usize] ^ MUL11[state[2] as usize];
    tmp[2] = MUL14[state[2] as usize] ^ MUL9[state[1] as usize] ^ MUL13[state[0] as usize] ^ MUL11[state[3] as usize];
    tmp[3] = MUL14[state[3] as usize] ^ MUL9[state[2] as usize] ^ MUL13[state[1] as usize] ^ MUL11[state[0] as usize];

    for i in 0..4 {
        state[i] = tmp[i];
    }
}

/* *
 * \brief           Internal AES block decryption function. This is only
 *                  exposed to allow overriding it using see
 *                  \c MBEDTLS_AES_DECRYPT_ALT.
 *
 * \param ctx       The AES context to use for decryption.
 * \param input     The ciphertext block.
 * \param output    The output (plaintext) block.
 *
 * \return          \c 0 on success.
 */
pub fn mbedtls_internal_aes_decrypt(
    ctx: &mbedtls_aes_context,
    input: &[u8; 16],
    mut output: &mut [u8; 16],
) -> isize {
    let mut round_key: [u8; 16] = [0; 16];

    for i in 0..16
    {
        output[i] = input[i];
        round_key[i] = ctx.expanded_key8[160 + i];
    }

    add_round_key(&mut output, &round_key);
    inv_shift_rows(&mut output);
    inv_sub_bytes(&mut output);

    for i in (0..ctx.nr-1).rev()
    {
        for j in 16 * (i + 1)..16 * (i + 2)
        {
            round_key[j - 16 * (i + 1)] = ctx.expanded_key8[j];
        }
        add_round_key(&mut output, &round_key);
        inv_mix_columns(&mut output);
        inv_shift_rows(&mut output);
        inv_sub_bytes(&mut output);
    }

    add_round_key(&mut output, &ctx.key8);
    0
}


/* *
 * \brief  This function performs an AES-CBC encryption or decryption operation
 *         on full blocks.
 *
 *         It performs the operation defined in the \p mode
 *         parameter (encrypt/decrypt), on the input data buffer defined in
 *         the \p input parameter.
 *
 *         It can be called as many times as needed, until all the input
 *         data is processed. mbedtls_aes_init(), and either
 *         mbedtls_aes_setkey_enc() or mbedtls_aes_setkey_dec() must be called
 *         before the first call to this API with the same context.
 *
 * \note   This function operates on full blocks, that is, the input size
 *         must be a multiple of the AES block size of \c 16 Bytes.
 *
 * \note   Upon exit, the content of the IV is updated so that you can
 *         call the same function again on the next
 *         block(s) of data and get the same result as if it was
 *         encrypted in one call. This allows a "streaming" usage.
 *         If you need to retain the contents of the IV, you should
 *         either save it manually or use the cipher module instead.
 *
 *
 * \param ctx      The AES context to use for encryption or decryption.
 *                 It must be initialized and bound to a key.
 * \param mode     The AES operation: #MBEDTLS_AES_ENCRYPT or
 *                 #MBEDTLS_AES_DECRYPT.
 * \param length   The length of the input data in Bytes. This must be a
 *                 multiple of the block size (\c 16 Bytes).
 * \param iv       Initialization vector (updated after use).
 *                 It must be a readable and writeable buffer of \c 16 Bytes.
 * \param input    The buffer holding the input data.
 *                 It must be readable and of size \p length Bytes.
 * \param output   The buffer holding the output data.
 *                 It must be writeable and of size \p length Bytes.
 *
 * \return         \c 0 on success.
 * \return         #MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH
 *                 on failure.
 */
pub fn mbedtls_aes_crypt_cbc(
    ctx: &mbedtls_aes_context,
    mode: isize,
    mut length: usize,
    iv: &[u8; 16],
    input: &[u8],
    mut output: &mut [u8],
) -> isize {
    let mut temp_input: [u8; 16] = [0; 16];
    // let mut temp: [u8; 16] = [0; 16];
    let mut temp_output: [u8; 16] = [0; 16];
    let mut temp_iv: [u8; 16] = [0; 16];

    for i in 0..16
    {
        temp_iv[i] = iv[i];
    }

    if length % 16 != 0
    {
        return MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH;
    }

    if mode == MBEDTLS_AES_ENCRYPT
    {
        for i in (0..length).step_by(16)
        {
            for j in 0..16
            {
                temp_input[j] = input[i + j] ^ temp_iv[j];
                // temp[j] = input[i + j];
            }
            mbedtls_internal_aes_encrypt(&ctx, &temp_input, &mut temp_output);

            for j in 0..16
            {
                output[i + j] = temp_output[j];
                // output[i + j] ^= iv[j];
                temp_iv[j] = temp_output[j];
            }
        }
    }

    else
    {
        for i in (0..length).step_by(16)
        {
            for j in 0..16
            {
                // output[i + j] = input[i + j] ^ iv[j];
                temp_input[j] = input[i + j];
            }

            mbedtls_internal_aes_decrypt(&ctx, &temp_input, &mut temp_output);
            for j in 0..16
            {
                output[i + j] = temp_output[j] ^ temp_iv[j];
                temp_iv[j] = temp_input[j];
            }
        }
    }
    0
}



/* *
 * \brief       This function performs an AES-OFB (Output Feedback Mode)
 *              encryption or decryption operation.
 *
 *              For OFB, you must set up the context with
 *              mbedtls_aes_setkey_enc(), regardless of whether you are
 *              performing an encryption or decryption operation. This is
 *              because OFB mode uses the same key schedule for encryption and
 *              decryption.
 *
 *              The OFB operation is identical for encryption or decryption,
 *              therefore no operation mode needs to be specified.
 *
 * \note        Upon exit, the content of iv, the Initialisation Vector, is
 *              updated so that you can call the same function again on the next
 *              block(s) of data and get the same result as if it was encrypted
 *              in one call. This allows a "streaming" usage, by initialising
 *              iv_off to 0 before the first call, and preserving its value
 *              between calls.
 *
 *              For non-streaming use, the iv should be initialised on each call
 *              to a unique value, and iv_off set to 0 on each call.
 *
 *              If you need to retain the contents of the initialisation vector,
 *              you must either save it manually or use the cipher module
 *              instead.
 *
 * \warning     For the OFB mode, the initialisation vector must be unique
 *              every encryption operation. Reuse of an initialisation vector
 *              will compromise security.
 *
 * \param ctx      The AES context to use for encryption or decryption.
 *                 It must be initialized and bound to a key.
 * \param length   The length of the input data.
 * \param iv_off   The offset in IV (updated after use).
 *                 It must point to a valid \c size_t.
 * \param iv       The initialization vector (updated after use).
 *                 It must be a readable and writeable buffer of \c 16 Bytes.
 * \param input    The buffer holding the input data.
 *                 It must be readable and of size \p length Bytes.
 * \param output   The buffer holding the output data.
 *                 It must be writeable and of size \p length Bytes.
 *
 * \return         \c 0 on success.
 */
pub fn mbedtls_aes_crypt_ofb(
    ctx: &mbedtls_aes_context,
    length: usize,
    mut iv_off: &mut usize,
    iv: &mut [u8; 16],
    input: &[u8],
    mut output: &mut [u8],
) -> isize {

    let mut temp_iv_input: [u8; 16] = [0; 16];
    let mut temp_iv_output: [u8; 16] = [0; 16];

    for i in 0..16
    {
        temp_iv_input[i] = iv[i];
        temp_iv_output[i] = iv[i];
    }


    if *iv_off > 15 || length % 16 != 0
    {
        return MBEDTLS_ERR_AES_BAD_INPUT_DATA;
    }

    for i in (0..length).step_by(16)
    {
        if *iv_off == 0
        {
            let ret = mbedtls_aes_crypt_ecb(&ctx, MBEDTLS_AES_ENCRYPT, &temp_iv_input, &mut temp_iv_output);
            if ret != 0
            {
                return ret;
            }
        }
        for j in 0..16
        {
            output[i + j] = input[i + j] ^ temp_iv_output[*iv_off];
            iv[j] = temp_iv_output[j];
            temp_iv_input[j] = temp_iv_output[j];
            *iv_off = (*iv_off + 1) & 0x0F;
        }
    }
    0
}


//====================================================================================
/*
/**
 * \brief      This function performs an AES-XTS encryption or decryption
 *             operation for an entire XTS data unit.
 *
 *             AES-XTS encrypts or decrypts blocks based on their location as
 *             defined by a data unit number. The data unit number must be
 *             provided by \p data_unit.
 *
 *             NIST SP 800-38E limits the maximum size of a data unit to 2^20
 *             AES blocks. If the data unit is larger than this, this function
 *             returns #MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH.
 *
 * \param ctx          The AES XTS context to use for AES XTS operations.
 *                     It must be initialized and bound to a key.
 * \param mode         The AES operation: #MBEDTLS_AES_ENCRYPT or
 *                     #MBEDTLS_AES_DECRYPT.
 * \param length       The length of a data unit in Bytes. This can be any
 *                     length between 16 bytes and 2^24 bytes inclusive
 *                     (between 1 and 2^20 block cipher blocks).
 * \param data_unit    The address of the data unit encoded as an array of 16
 *                     bytes in little-endian format. For disk encryption, this
 *                     is typically the index of the block device sector that
 *                     contains the data.
 * \param input        The buffer holding the input data (which is an entire
 *                     data unit). This function reads \p length Bytes from \p
 *                     input.
 * \param output       The buffer holding the output data (which is an entire
 *                     data unit). This function writes \p length Bytes to \p
 *                     output.
 *
 * \return             \c 0 on success.
 * \return             #MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH if \p length is
 *                     smaller than an AES block in size (16 Bytes) or if \p
 *                     length is larger than 2^20 blocks (16 MiB).
 */
fn mbedtls_aes_crypt_xts(
    ctx: &mut mbedtls_aes_xts_context,
    mode: isize,
    length: usize,
    data_unit: [char; 16],
    input: &char,
    output: &mut char,
) -> isize {
    1
}

/**
 * \brief This function performs an AES-CFB128 encryption or decryption
 *        operation.
 *
 *        It performs the operation defined in the \p mode
 *        parameter (encrypt or decrypt), on the input data buffer
 *        defined in the \p input parameter.
 *
 *        For CFB, you must set up the context with mbedtls_aes_setkey_enc(),
 *        regardless of whether you are performing an encryption or decryption
 *        operation, that is, regardless of the \p mode parameter. This is
 *        because CFB mode uses the same key schedule for encryption and
 *        decryption.
 *
 * \note  Upon exit, the content of the IV is updated so that you can
 *        call the same function again on the next
 *        block(s) of data and get the same result as if it was
 *        encrypted in one call. This allows a "streaming" usage.
 *        If you need to retain the contents of the
 *        IV, you must either save it manually or use the cipher
 *        module instead.
 *
 *
 * \param ctx      The AES context to use for encryption or decryption.
 *                 It must be initialized and bound to a key.
 * \param mode     The AES operation: #MBEDTLS_AES_ENCRYPT or
 *                 #MBEDTLS_AES_DECRYPT.
 * \param length   The length of the input data in Bytes.
 * \param iv_off   The offset in IV (updated after use).
 *                 It must point to a valid \c size_t.
 * \param iv       The initialization vector (updated after use).
 *                 It must be a readable and writeable buffer of \c 16 Bytes.
 * \param input    The buffer holding the input data.
 *                 It must be readable and of size \p length Bytes.
 * \param output   The buffer holding the output data.
 *                 It must be writeable and of size \p length Bytes.
 *
 * \return         \c 0 on success.
 */
fn mbedtls_aes_crypt_cfb128(
    ctx: &mut mbedtls_aes_context,
    mode: isize,
    length: usize,
    iv_off: &usize,
    iv: [char; 16],
    input: &char,
    output: &char,
) -> isize {
    1
}

/**
 * \brief This function performs an AES-CFB8 encryption or decryption
 *        operation.
 *
 *        It performs the operation defined in the \p mode
 *        parameter (encrypt/decrypt), on the input data buffer defined
 *        in the \p input parameter.
 *
 *        Due to the nature of CFB, you must use the same key schedule for
 *        both encryption and decryption operations. Therefore, you must
 *        use the context initialized with mbedtls_aes_setkey_enc() for
 *        both #MBEDTLS_AES_ENCRYPT and #MBEDTLS_AES_DECRYPT.
 *
 * \note  Upon exit, the content of the IV is updated so that you can
 *        call the same function again on the next
 *        block(s) of data and get the same result as if it was
 *        encrypted in one call. This allows a "streaming" usage.
 *        If you need to retain the contents of the
 *        IV, you should either save it manually or use the cipher
 *        module instead.
 *
 *
 * \param ctx      The AES context to use for encryption or decryption.
 *                 It must be initialized and bound to a key.
 * \param mode     The AES operation: #MBEDTLS_AES_ENCRYPT or
 *                 #MBEDTLS_AES_DECRYPT
 * \param length   The length of the input data.
 * \param iv       The initialization vector (updated after use).
 *                 It must be a readable and writeable buffer of \c 16 Bytes.
 * \param input    The buffer holding the input data.
 *                 It must be readable and of size \p length Bytes.
 * \param output   The buffer holding the output data.
 *                 It must be writeable and of size \p length Bytes.
 *
 * \return         \c 0 on success.
 */
fn mbedtls_aes_crypt_cfb8(
    ctx: &mut mbedtls_aes_context,
    mode: isize,
    length: usize,
    iv: [char; 16],
    input: &char,
    output: &char,
) -> isize {
    1
}



/**
 * \brief      This function performs an AES-CTR encryption or decryption
 *             operation.
 *
 *             This function performs the operation defined in the \p mode
 *             parameter (encrypt/decrypt), on the input data buffer
 *             defined in the \p input parameter.
 *
 *             Due to the nature of CTR, you must use the same key schedule
 *             for both encryption and decryption operations. Therefore, you
 *             must use the context initialized with mbedtls_aes_setkey_enc()
 *             for both #MBEDTLS_AES_ENCRYPT and #MBEDTLS_AES_DECRYPT.
 *
 * \warning    You must never reuse a nonce value with the same key. Doing so
 *             would void the encryption for the two messages encrypted with
 *             the same nonce and key.
 *
 *             There are two common strategies for managing nonces with CTR:
 *
 *             1. You can handle everything as a single message processed over
 *             successive calls to this function. In that case, you want to
 *             set \p nonce_counter and \p nc_off to 0 for the first call, and
 *             then preserve the values of \p nonce_counter, \p nc_off and \p
 *             stream_block across calls to this function as they will be
 *             updated by this function.
 *
 *             With this strategy, you must not encrypt more than 2**128
 *             blocks of data with the same key.
 *
 *             2. You can encrypt separate messages by dividing the \p
 *             nonce_counter buffer in two areas: the first one used for a
 *             per-message nonce, handled by yourself, and the second one
 *             updated by this function internally.
 *
 *             For example, you might reserve the first 12 bytes for the
 *             per-message nonce, and the last 4 bytes for internal use. In that
 *             case, before calling this function on a new message you need to
 *             set the first 12 bytes of \p nonce_counter to your chosen nonce
 *             value, the last 4 to 0, and \p nc_off to 0 (which will cause \p
 *             stream_block to be ignored). That way, you can encrypt at most
 *             2**96 messages of up to 2**32 blocks each with the same key.
 *
 *             The per-message nonce (or information sufficient to reconstruct
 *             it) needs to be communicated with the ciphertext and must be unique.
 *             The recommended way to ensure uniqueness is to use a message
 *             counter. An alternative is to generate random nonces, but this
 *             limits the number of messages that can be securely encrypted:
 *             for example, with 96-bit random nonces, you should not encrypt
 *             more than 2**32 messages with the same key.
 *
 *             Note that for both stategies, sizes are measured in blocks and
 *             that an AES block is 16 bytes.
 *
 * \warning    Upon return, \p stream_block contains sensitive data. Its
 *             content must not be written to insecure storage and should be
 *             securely discarded as soon as it's no longer needed.
 *
 * \param ctx              The AES context to use for encryption or decryption.
 *                         It must be initialized and bound to a key.
 * \param length           The length of the input data.
 * \param nc_off           The offset in the current \p stream_block, for
 *                         resuming within the current cipher stream. The
 *                         offset pointer should be 0 at the start of a stream.
 *                         It must point to a valid \c size_t.
 * \param nonce_counter    The 128-bit nonce and counter.
 *                         It must be a readable-writeable buffer of \c 16 Bytes.
 * \param stream_block     The saved stream block for resuming. This is
 *                         overwritten by the function.
 *                         It must be a readable-writeable buffer of \c 16 Bytes.
 * \param input            The buffer holding the input data.
 *                         It must be readable and of size \p length Bytes.
 * \param output           The buffer holding the output data.
 *                         It must be writeable and of size \p length Bytes.
 *
 * \return                 \c 0 on success.
 */
fn mbedtls_aes_crypt_ctr(
    ctx: &mut mbedtls_aes_context,
    length: usize,
    nc_off: &usize,
    nonce_counter: [char; 16],
    stream_block: [char; 16],
    input: &char,
    output: &char,
) -> isize {
    1
}



/**
 * \brief           Deprecated internal AES block encryption function
 *                  without return value.
 *
 * \deprecated      Superseded by mbedtls_internal_aes_encrypt()
 *
 * \param ctx       The AES context to use for encryption.
 * \param input     Plaintext block.
 * \param output    Output (ciphertext) block.
 */
fn mbedtls_aes_encrypt(ctx: &mut mbedtls_aes_context, input: [char; 16], output: [char; 16]) {}

/**
 * \brief           Deprecated internal AES block decryption function
 *                  without return value.
 *
 * \deprecated      Superseded by mbedtls_internal_aes_decrypt()
 *
 * \param ctx       The AES context to use for decryption.
 * \param input     Ciphertext block.
 * \param output    Output (plaintext) block.
 */
fn mbedtls_aes_decrypt(ctx: &mut mbedtls_aes_context, input: [char; 16], output: [char; 16]) {}

/**
 * \brief          Checkup routine.
 *
 * \return         \c 0 on success.
 * \return         \c 1 on failure.
 */
fn mbedtls_aes_self_test(verbose: isize) -> isize {
    1
}


*/