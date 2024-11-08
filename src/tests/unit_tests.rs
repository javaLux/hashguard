#[cfg(test)]
pub mod unit_tests {
    use crate::{filename_handling, os_specifics, utils, verify};

    #[test]
    fn test_os_type() {
        use os_specifics::{get_os, OS};
        let os = get_os();

        match os {
            Some(OS::Linux) => assert_eq!(Some(OS::Linux), os),
            Some(OS::MacOs) => assert_eq!(Some(OS::MacOs), os),
            Some(OS::Windows) => assert_eq!(Some(OS::Windows), os),
            None => assert_eq!(None, os),
        }
    }

    #[test]
    fn test_lower_hex() {
        use verify::is_lower_hex;
        let check_sum = "c92fae5e42b9aecf444a66c8ec563c652f60b1e231dfdd33e";
        assert!(is_lower_hex(check_sum));
    }

    #[test]
    fn test_upper_hex() {
        use verify::is_upper_hex;
        let check_sum = "A92fAE5G42B9F444";
        assert!(is_upper_hex(check_sum));
    }

    #[test]
    fn test_filenames_windows() {
        use filename_handling::is_filename_valid_on_windows;

        let filename1 = "valid_filename.txt";
        let filename2 = "test/filename.txt";
        let filename3 = "file?name.pdf";
        let filename4 = "filename\\.csv";

        assert!(is_filename_valid_on_windows(filename1));
        assert!(!is_filename_valid_on_windows(filename2));
        assert!(!is_filename_valid_on_windows(filename3));
        assert!(!is_filename_valid_on_windows(filename4));
    }

    #[test]
    fn test_reserved_filename_on_windows() {
        use filename_handling::is_reserved_filename_on_windows;

        let reserved_filenames = vec![
            "CON",
            "PRN",
            "AUX",
            "NUL",
            "COM1",
            "COM2",
            "COM3",
            "COM4",
            "COM5",
            "COM6",
            "COM7",
            "COM8",
            "COM9",
            "LPT1",
            "LPT2",
            "LPT3",
            "LPT4",
            "LPT5",
            "LPT6",
            "LPT7",
            "LPT8",
            "LPT9",
            "CON.txt",
            "PRN.docs",
            "AUX.toml",
            "NUL.cu",
            "COM1.bin",
            "COM2.test",
            "COM3.zip",
            "COM4.7z",
            "COM5.op",
            "COM6.exe",
            "COM7.sh",
            "COM8.rs",
            "COM9.lil",
        ];

        for filename in reserved_filenames {
            assert!(is_reserved_filename_on_windows(filename));
        }
    }

    #[test]
    fn trim_dot_from_end() {
        let test_string = "Hello world.";
        let result = test_string.trim_end_matches(&['.']);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_filenames_unix() {
        use filename_handling::is_filename_valid_on_unix;

        let filename1 = "valid_filename";
        let filename2 = "test/filename.txt";
        let filename3 = "file:name.pdf";
        let filename4 = "filename\\";

        assert!(is_filename_valid_on_unix(filename1));
        assert!(!is_filename_valid_on_unix(filename2));
        assert!(!is_filename_valid_on_unix(filename3));
        assert!(!is_filename_valid_on_unix(filename4));
    }

    #[test]
    fn test_valid_url_1() {
        use utils::is_valid_url;

        let test_url = "http://example.com/files/document.pdf";

        assert!(is_valid_url(test_url));
    }

    #[test]
    fn test_valid_url_2() {
        use utils::is_valid_url;

        let test_url = "https://google.de";

        assert!(is_valid_url(test_url));
    }

    #[test]
    fn test_invalid_url_1() {
        use utils::is_valid_url;

        let test_url = "HelloWorld";

        assert!(!is_valid_url(test_url));
    }

    #[test]
    fn test_invalid_url_2() {
        use utils::is_valid_url;

        let test_url = "file://tmp/foo";

        assert!(!is_valid_url(test_url));
    }

    #[test]
    fn test_invalid_url_3() {
        use utils::is_valid_url;
        let test_url = "www.example.com";
        assert!(!is_valid_url(test_url));
    }

    #[test]
    fn test_extract_filename_from_url_1() {
        use utils::extract_file_name_from_url;

        let test_url = "https://example.com/files/document.pdf";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, Some("document.pdf".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_2() {
        use utils::extract_file_name_from_url;

        let test_url = "http://blah.com/path1/path2/test_file.txt?a=1&b=2";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, Some("test_file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_3() {
        use utils::extract_file_name_from_url;

        let test_url = "https://google.de/";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, None);
    }

    #[test]
    fn test_content_disposition_filename_1() {
        use utils::content_disposition_filename;

        let header_value = "attachment; filename=my_test_file.txt";

        assert_eq!(
            Some("my_test_file.txt".to_string()),
            content_disposition_filename(header_value)
        );
    }

    #[test]
    fn test_content_disposition_filename_2() {
        use utils::content_disposition_filename;

        let header_value = r#"attachment; filename="my_test_file.txt""#;

        assert_eq!(
            Some("my_test_file.txt".to_string()),
            content_disposition_filename(header_value)
        );
    }

    #[test]
    fn test_content_disposition_filename_3() {
        use utils::content_disposition_filename;

        let header_value = "attachment; filename*=UTF-8''my_test_file.txt";

        assert_eq!(
            Some("my_test_file.txt".to_string()),
            content_disposition_filename(header_value)
        );
    }

    #[test]
    fn test_content_disposition_filename_4() {
        use utils::content_disposition_filename;

        let header_value = r#"attachment; filename*=utf-8''my_test_file.txt"#;

        assert_eq!(
            Some("my_test_file.txt".to_string()),
            content_disposition_filename(header_value)
        );
    }

    #[test]
    fn test_content_disposition_filename_5() {
        use utils::content_disposition_filename;

        let header_value = "attachment; filename*=utf-8''Na%C3%AFve%20file.txt";

        assert_eq!(
            Some("Na%C3%AFve%20file.txt".to_string()),
            content_disposition_filename(header_value)
        );
    }

    #[test]
    fn test_content_disposition_filename_empty() {
        use utils::content_disposition_filename;

        let header_value = "";
        assert_eq!(content_disposition_filename(header_value), None);
    }

    #[test]
    fn test_content_disposition_filename_no_filename() {
        use utils::content_disposition_filename;

        let header_value = "attachment; other_param=test";
        assert_eq!(content_disposition_filename(header_value), None);
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_empty_input() {
        use utils::decode_percent_encoded_to_utf_8;

        let input = "";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_no_encoding() {
        use utils::decode_percent_encoded_to_utf_8;

        let input = "example_filename.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "example_filename.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_single_encoding() {
        use utils::decode_percent_encoded_to_utf_8;

        let input = "file%20with%20spaces.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "file with spaces.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_multiple_encodings_1() {
        use utils::decode_percent_encoded_to_utf_8;

        let input = "file%20with%20spaces%20and%20special%21%23%25.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "file with spaces and special!#%.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_multiple_encodings_2() {
        use utils::decode_percent_encoded_to_utf_8;

        let input = "Na%C3%AFve%20file.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "Naïve file.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_invalid_encoding() {
        use utils::decode_percent_encoded_to_utf_8;

        // Invalid percent encoding, should be treated as plain text
        let input = "invalid%2xencoding";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "invalid%2xencoding");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_linux() {
        use os_specifics::OS;
        use utils::replace_invalid_chars_with_underscore;

        let filename = "my:file/with\\invalid\\characters.txt";
        let os_type = OS::Linux;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "my_file_with_invalid_characters.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_macos() {
        use os_specifics::OS;
        use utils::replace_invalid_chars_with_underscore;

        let filename = "my:file/with\\invalid\\characters.txt";
        let os_type = OS::MacOs;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "my_file_with_invalid_characters.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_windows() {
        use os_specifics::OS;
        use utils::replace_invalid_chars_with_underscore;

        let filename = "my?file*with<invalid>characters\\fancy:style.txt";
        let os_type = OS::Windows;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "my_file_with_invalid_characters_fancy_style.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_no_replacement() {
        use os_specifics::OS;
        use utils::replace_invalid_chars_with_underscore;

        let filename = "file_without_invalid_characters.txt";
        let os_type = OS::Linux;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "file_without_invalid_characters.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_empty_filename() {
        use os_specifics::OS;
        use utils::replace_invalid_chars_with_underscore;

        let filename = "";
        let os_type = OS::Windows;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "");
    }

    #[test]
    fn test_redirect_status_codes() {
        use utils::is_redirection;

        let redirect_codes = [301, 302, 307, 308];

        for status_code in redirect_codes {
            assert!(is_redirection(status_code));
        }

        assert!(!is_redirection(404));
    }
}
