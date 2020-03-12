const VERSION_LEGACY: u8 = 1 << 0;
const VERSION_STING: u8 = 1 << 1;
pub(crate) const SUPPORTED_VERSIONS: [u8; 1] = [VERSION_LEGACY | VERSION_STING];

enum SupportedVersion {
    CommonSupportedVersion(u8),
    UnsupportedVersion(u8),
}

fn common_supported_version(own_supported_versions: &[u8], supported_versions: &[u8]) -> SupportedVersion {
    let mut highest_supported_version: u8 = 0;
    let mut highest_supported_version_by_neighbor: u8 = 0;

    for (i, ownSupportedVersion) in own_supported_versions.iter().enumerate() {
        // max check up to advertised versions by the neighbor
        if i >= supported_versions.len() {
            break;
        }

        // get versions matched by both
        let supported: u8 = supported_versions[i] & ownSupportedVersion;
        // println!("{:?}", supported);

        // none supported
        if supported == 0 {
            continue;
        }

        // iterate through all bits and find highest (more to the left is higher)
        let mut highest: u8 = 0;
        for j in 0..8 {
            if ((supported >> j) & 1) == 1 {
                highest = j + 1;
            }
        }

        highest_supported_version = highest + ((i as u8) * 8);
    }

    // if the highest version is still 0, it means that we don't support any protocol version the
    // neighbor supports
    if highest_supported_version == 0 {
        // grab last byte denoting the highest versions.
        // a node will only hold version bytes if at least one version in that
        // byte is supported, therefore it's safe to assume, that the last byte contains
        // the highest supported version of a given node.
        //
        // TODO unwrap ?
        let lastVersionsByte: &u8 = supported_versions.last().unwrap();

        // iterate through all bits and find highest (more to the left is higher)
        let mut highest: u8 = 0;
        for j in 0..8 {
            if ((lastVersionsByte >> j) & 1) == 1 {
                highest = j + 1;
            }
        }
        highest_supported_version_by_neighbor = highest + ((supported_versions.len() - 1) as u8 * 8);
        return SupportedVersion::UnsupportedVersion(highest_supported_version_by_neighbor);
        // 	return highest_supported_version_by_neighbor, ErrVersionNotSupported
    }

    SupportedVersion::CommonSupportedVersion(highest_supported_version)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        for i in 0..32 {
            let mut byte = 1;

            for j in 0..8 {
                let mut version = [0; 32];
                let expected_version = i as u16 * 8 + j as u16 + 1;

                version[i] = byte;
                if expected_version != 256 {
                    match common_supported_version(&version, &version) {
                        SupportedVersion::CommonSupportedVersion(v) => assert_eq!(v, expected_version as u8),
                        _ => unreachable!(),
                    }
                }
                byte = byte << 1;
            }
        }
    }
}
