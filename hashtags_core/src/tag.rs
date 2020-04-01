use std::result::Result;
use std::vec::Vec;
use super::error::Error;
use regex::Regex;

pub fn extract_tags(note: &str) -> Result<Vec<&str>, Error> {
    let re = Regex::new(r"((^|\s)#[^\s\t\.\?#,]+)").unwrap();
    let mut tags = Vec::<&str>::new();
    for m in re.find_iter(note) {
        let start = match note[m.start()..].find("#") {
            Some(i) => m.start()+i+1,
            None => return Err(Error::GenericError(format!("unable to find # {}", note)))
        };
        tags.push(&note[start..m.end()]);
    }
    if tags.len() == 0 {
        return Err(Error::GenericError(format!("no tags extracted: {}", note)))
    }
    // Remove duplications.
    tags.sort();
    tags.dedup();
    Ok(tags)
}

#[cfg(test)]
mod test {
    use super::extract_tags;

    #[test]
    fn test_basic() {
        assert!(extract_tags("kdfkjsdkfjsf").is_err());
        assert_eq!(extract_tags("ss #ss #tt # sdkjfk #yy").unwrap(), vec!["ss", "tt", "yy"]);
        assert_eq!(extract_tags("#ss").unwrap(), vec!["ss"]);
        assert_eq!(extract_tags("#ss #tt #ss").unwrap(), vec!["ss", "tt"]);
    }

    #[test]
    fn test_utf8() {
        assert!(extract_tags("我家門前有小河").is_err());
        assert_eq!(extract_tags("ss #測試 #哎呦 # sdkjfk #幹嘛").unwrap(), vec!["哎呦", "幹嘛", "測試"]);
        assert_eq!(extract_tags("#再測").unwrap(), vec!["再測"]);
        assert_eq!(extract_tags("ss #幹嘛 #測試  # sdkjfk #幹嘛").unwrap(), vec!["幹嘛", "測試"]);
    }
}
