#[macro_export]
macro_rules! out_yarn {
    ($($arg:tt)*) => {
        {
            print!("[YARN] ");
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! out_info {
    ($($arg:tt)*) => {
        {
            print!("[INFO] ");
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! out_fix {
    ($($arg:tt)*) => {
        {
            print!("[FIX!] ");
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! out_npm {
    ($($arg:tt)*) => {
        {
            print!("[NPM?] ");
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! out_indent {
    ($($arg:tt)*) => {
        {
            print!("    ");
            println!($($arg)*);
        }
    };
}
