pub const PF_PLUG_IN_VERSION: u16 = 13;
pub const PF_PLUG_IN_SUBVERS: u16 = 28;

fn hex(s: &'static str) -> Vec<u8> {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .into_bytes()
        .chunks(2)
        .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
        .collect::<Vec<u8>>()
}

fn ensure_eq(our: &[u8], expected: &'static str) {
    let expected = hex(expected);
    let pretty_our = pretty_hex::pretty_hex(&our);
    let pretty_expected = pretty_hex::pretty_hex(&expected);
    similar_asserts::assert_eq!(pretty_expected, pretty_our);
}

use pipl::*;

#[test]
#[cfg(target_os = "macos")]
fn skeleton_macos() {
    let pipl = build_pipl(vec![
        Property::Kind(PIPLType::AEEffect),
        Property::Name("Portable"),
        Property::Category("Sample Plug-ins"),
        //Property::CodeWin64X86("EffectMain"),
        Property::CodeMacIntel64("EffectMain"),
        Property::CodeMacARM64("EffectMain"),
        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version {
            major: PF_PLUG_IN_VERSION,
            minor: PF_PLUG_IN_SUBVERS,
        },
        Property::AE_Effect_Version {
            version: 1,
            subversion: 1,
            bugversion: 0,
            stage: Stage::Develop,
            build: 1,
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(OutFlags::PixIndependent | OutFlags::UseOutputExtent),
        Property::AE_Effect_Global_OutFlags_2(OutFlags2::SupportsThreadedRendering),
        Property::AE_Effect_Match_Name("ADBE Portable"),
        Property::AE_Reserved_Info(0),
        Property::AE_Effect_Support_URL("https://www.adobe.com"),
    ])
    .unwrap();

    ensure_eq(&pipl, "000000000000000E3842494D6B696E64000000000000000465464B543842494D6E616D65000000000000000908506F727461626C650000003842494D6361746700000000000000100F53
                      616D706C6520506C75672D696E733842494D6D693634000000000000000B0A4566666563744D61696E003842494D6D613634000000000000000B0A4566666563744D61696E003842494D
                      655056520000000000000004000200003842494D655356520000000000000004000D001C3842494D655645520000000000000004000880013842494D65494E4600000000000000020000
                      00003842494D65474C4F0000000000000004000004403842494D65474C320000000000000004080000003842494D654D4E41000000000000000E0D4144424520506F727461626C650000
                      3842494D6165464C0000000000000004000000003842494D6555524C00000000000000161568747470733A2F2F7777772E61646F62652E636F6D0000");

    #[rustfmt::skip]
    let rsrc = create_rsrc(&[
        (b"PiPL", &[
            (16000, &pipl)
        ])
    ]).unwrap();

    let hex_data = "0000010000000268000001680000003200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
                    0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
                    0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
                    0000000000000000000000000000000000000000000000000000000000000000000000000164000000000000000e3842494d6b696e64000000000000000465464b543842494d6e616d65
                    000000000000000908506f727461626c650000003842494d6361746700000000000000100f53616d706c6520506c75672d696e733842494d6d693634000000000000000b0a4566666563
                    744d61696e003842494d6d613634000000000000000b0a4566666563744d61696e003842494d655056520000000000000004000200003842494d655356520000000000000004000d001c
                    3842494d655645520000000000000004000880013842494d65494e460000000000000002000000003842494d65474c4f0000000000000004000004403842494d65474c32000000000000
                    0004080000003842494d654d4e41000000000000000e0d4144424520506f727461626c6500003842494d6165464c0000000000000004000000003842494d6555524c0000000000000016
                    1568747470733a2f2f7777772e61646f62652e636f6d000000000100000002680000016800000032000000000a000000001c003200005069504c0000000a3e80ffff0000000001000000";
    ensure_eq(&rsrc, hex_data);
}

#[test]
#[cfg(target_os = "windows")]
fn sdk_invert_proc_amp() {
    #[rustfmt::skip]
    let pipl = build_pipl(vec![
        Property::Kind(PIPLType::AEEffect),
        Property::Name("SDK_Invert_ProcAmp"),
        Property::Category("Sample Plug-ins"),
        Property::CodeWin64X86("EffectMain"),
        // Property::CodeMacIntel64("EffectMain"),
        // Property::CodeMacARM64("EffectMain"),
        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version { major: PF_PLUG_IN_VERSION, minor: PF_PLUG_IN_SUBVERS },
        Property::AE_Effect_Version {
            version: 1,
            subversion: 1,
            bugversion: 0,
            stage: Stage::Develop,
            build: 1
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(
			OutFlags::PixIndependent |
			OutFlags::DeepColorAware
		),
        Property::AE_Effect_Global_OutFlags_2(
            OutFlags2::FloatColorAware |
            OutFlags2::SupportsSmartRender |
            OutFlags2::SupportsThreadedRendering |
            OutFlags2::SupportsGpuRenderF32,
        ),
        Property::AE_Effect_Match_Name("ADBE SDK_Invert_ProcAmp"),
        Property::AE_Reserved_Info(0),
        Property::AE_Effect_Support_URL("https://www.adobe.com"),
    ]).unwrap();

    ensure_eq(&pipl, "0100000000000D0000004D494238646E696B0000000004000000544B46654D494238656D616E00000000140000001253444B5F496E766572745F50726F63416D70004D49423867746163
                      00000000100000000F53616D706C6520506C75672D696E734D49423834363638000000000C0000004566666563744D61696E00004D494238525650650000000004000000020000004D49
                      42385256536500000000040000000D001C004D494238524556650000000004000000018008004D494238464E49650000000004000000000000004D4942384F4C47650000000004000000
                      000400024D494238324C476500000000040000000014000A4D494238414E4D65000000001800000017414442452053444B5F496E766572745F50726F63416D704D4942384C4665610000
                      000004000000000000004D4942384C52556500000000180000001568747470733A2F2F7777772E61646F62652E636F6D0000");
}

#[test]
fn creating_rsrc() {
    let pipl = hex("000000000000000d3842494d6b696e64000000000000000465464b543842494d6e616d65000000000000001110526564756365204e6f697365207635200000003842494d63617467000000000
                    000000b0a4e65617420566964656f003842494d6d693634000000000000000f0e456e747279506f696e7446756e63003842494d6d613634000000000000000f0e456e747279506f696e7446756
                    e63003842494d655056520000000000000004000200003842494d655356520000000000000004000d001c3842494d655645520000000000000004002000013842494d65494e460000000000000
                    002000000003842494d65474c4f0000000000000004020000323842494d65474c32000000000000000408b294093842494d654d4e41000000000000000c0b4e656174566964656f35413842494
                    d6165464c000000000000000400000008");

    #[rustfmt::skip]
    let rsrc = create_rsrc(&[
        (b"PiPL", &[
            (16001, &pipl)
        ])
    ]).unwrap();

    ensure_eq(&rsrc, "0000010000000248000001480000003200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
                      0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
                      0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
                      0000000000000000000000000000000000000000000000000000000000000000000000000144000000000000000D3842494D6B696E64000000000000000465464B543842494D6E616D65
                      000000000000001110526564756365204E6F697365207635200000003842494D63617467000000000000000B0A4E65617420566964656F003842494D6D693634000000000000000F0E45
                      6E747279506F696E7446756E63003842494D6D613634000000000000000F0E456E747279506F696E7446756E63003842494D655056520000000000000004000200003842494D65535652
                      0000000000000004000D001C3842494D655645520000000000000004002000013842494D65494E460000000000000002000000003842494D65474C4F0000000000000004020000323842
                      494D65474C32000000000000000408B294093842494D654D4E41000000000000000C0B4E656174566964656F35413842494D6165464C0000000000000004000000080000010000000248
                      00000148000000320000000000000000001C003200005069504C0000000A3E81FFFF0000000000000000");
}
