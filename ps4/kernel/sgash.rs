/* kernel::sgash.rs */

use core::*;
use core::str::*;
use core::option::{Some, Option, None}; // Match statement
use core::iter::Iterator;
use kernel::*;
use super::super::platform::*;
use kernel::memory::Allocator;

pub static mut buffer: strings::cstr = strings::cstr {
				p: 0 as *mut u8,
				p_cstr_i: 0,
				max: 0
			      };

pub fn putchar(key: char) {
    unsafe {
	/*
	 * We need to include a blank asm call to prevent rustc
	 * from optimizing this part out
	 */
	asm!("");
	io::write_char(key, io::UART0);
    }
}

fn putstr(msg: &str) {
    for c in slice::iter(as_bytes(msg)) {
	putchar(*c as char);
    }
}

pub unsafe fn drawstr(msg: &str) {
    let old_fg = super::super::io::FG_COLOR;
    let mut x: u32 = 0x6699AAFF;
    for c in slice::iter(as_bytes(msg)) {
	x = (x << 8) + (x >> 24); 
	super::super::io::set_fg(x);
	drawchar(*c as char);
    }
    super::super::io::set_fg(old_fg);
}

pub unsafe fn putcstr(s: strings::cstr)
{
    let mut p = s.p as uint;
    while *(p as *char) != '\0'
    {
	putchar(*(p as *char));
	p += 1;
    }
}

pub unsafe fn parsekey(x: char) {
	let x = x as u8;
	// Set this to false to learn the keycodes of various keys!
	// Key codes are printed backwards because life is hard
		
	if (true) {
		match x { 
			13		=>	{ 
						parse();
						prompt(false); 
			}
			127		=>	{ 
				if (buffer.delete_char()) { 
					putchar('');
					putchar(' ');
					putchar(''); 
					backspace();
				}
			}
			_		=>	{ 
				if (buffer.add_char(x)) { 
					putchar(x as char);
					drawchar(x as char);
				}
			}
		}
	}
	else {
		keycode(x);
	}
}

unsafe fn drawchar(x: char)
{
	if x == '\n' {
		io::CURSOR_Y += io::CURSOR_HEIGHT;
		io::CURSOR_X = 0u32;
		return;
	}

    io::restore();
    io::draw_char(x);
    io::CURSOR_X += io::CURSOR_WIDTH;
    if io::CURSOR_X >= io::SCREEN_WIDTH {io::CURSOR_X -= io::SCREEN_WIDTH; io::CURSOR_Y += io::CURSOR_HEIGHT}
    io::backup();
    io::draw_cursor();
}

unsafe fn backspace()
{
    io::restore();
    io::CURSOR_X -= io::CURSOR_WIDTH;
    io::draw_char(' ');
    io::backup();
    io::draw_cursor();
}

fn keycode(x: u8) {
	let mut x = x;
	while  x != 0 {
		putchar((x%10+ ('0' as u8) ) as char);
		x = x/10;
	}
	putchar(' ');
}
fn screen() {
	
	putstr(&"\n                                                               "); 
	putstr(&"\n                                                               ");
	putstr(&"\n                       7=..~$=..:7                             "); 
	putstr(&"\n                  +$: =$$$+$$$?$$$+ ,7?                        "); 
	putstr(&"\n                  $$$$$$$$$$$$$$$$$$Z$$                        ");
	putstr(&"\n              7$$$$$$$$$$$$. .Z$$$$$Z$$$$$$                    ");
	putstr(&"\n           ~..7$$Z$$$$$7+7$+.?Z7=7$$Z$$Z$$$..:                 ");
	putstr(&"\n          ~$$$$$$$$7:     :ZZZ,     :7ZZZZ$$$$=                ");
	putstr(&"\n           Z$$$$$?                    .+ZZZZ$$                 ");
	putstr(&"\n       +$ZZ$$$Z7                         7ZZZ$Z$$I.            "); 
	putstr(&"\n        $$$$ZZZZZZZZZZZZZZZZZZZZZZZZI,    ,ZZZ$$Z              "); 
	putstr(&"\n      :+$$$$ZZZZZZZZZZZZZZZZZZZZZZZZZZZ=    $ZZ$$+~,           "); 
	putstr(&"\n     ?$Z$$$$ZZZZZZZZZZZZZZZZZZZZZZZZZZZZI   7ZZZ$ZZI           "); 
	putstr(&"\n      =Z$$+7Z$$7ZZZZZZZZ$$$$$$$ZZZZZZZZZZ  ~Z$?$ZZ?            ");	 
	putstr(&"\n    :$Z$Z...$Z  $ZZZZZZZ~       ~ZZZZZZZZ,.ZZ...Z$Z$~          "); 
	putstr(&"\n    7ZZZZZI$ZZ  $ZZZZZZZ~       =ZZZZZZZ7..ZZ$?$ZZZZ$          "); 
	putstr(&"\n      ZZZZ$:    $ZZZZZZZZZZZZZZZZZZZZZZ=     ~$ZZZ$:           "); 
	putstr(&"\n    7Z$ZZ$,     $ZZZZZZZZZZZZZZZZZZZZ7         ZZZ$Z$          "); 
	putstr(&"\n   =ZZZZZZ,     $ZZZZZZZZZZZZZZZZZZZZZZ,       ZZZ$ZZ+         "); 
	putstr(&"\n     ,ZZZZ,     $ZZZZZZZ:     =ZZZZZZZZZ     ZZZZZ$:           "); 
	putstr(&"\n    =$ZZZZ+     ZZZZZZZZ~       ZZZZZZZZ~   =ZZZZZZZI          "); 
	putstr(&"\n    $ZZ$ZZZ$$Z$$ZZZZZZZZZ$$$$   IZZZZZZZZZ$ZZZZZZZZZ$          "); 
	putstr(&"\n      :ZZZZZZZZZZZZZZZZZZZZZZ   ~ZZZZZZZZZZZZZZZZZ~            "); 
	putstr(&"\n     ,Z$$ZZZZZZZZZZZZZZZZZZZZ    ZZZZZZZZZZZZZZZZZZ~           "); 
	putstr(&"\n     =$ZZZZZZZZZZZZZZZZZZZZZZ     $ZZZZZZZZZZZZZZZ$+           "); 
	putstr(&"\n        IZZZZZ:.                        . ,ZZZZZ$              "); 
	putstr(&"\n       ~$ZZZZZZZZZZZ                 ZZZZ$ZZZZZZZ+             "); 
	putstr(&"\n           Z$ZZZ. ,Z~               =Z:.,ZZZ$Z                 "); 
	putstr(&"\n          ,ZZZZZ..~Z$.             .7Z:..ZZZZZ:                ");
	putstr(&"\n          ~7+:$ZZZZZZZZI=:.   .,=IZZZZZZZ$Z:=7=                ");
	putstr(&"\n              $$ZZZZZZZZZZZZZZZZZZZZZZ$ZZZZ                    ");
	putstr(&"\n              ==..$ZZZ$ZZZZZZZZZZZ$ZZZZ .~+                    "); 			
	putstr(&"\n                  I$?.?ZZZ$ZZZ$ZZZI =$7                        ");
	putstr(&"\n                       $7..I$7..I$,                            ");
	putstr(&"\n"); 
	putstr(&"\n _                     _     _                         _  ");
	putstr(&"\n| |                   (_)   | |                       | | ");
	putstr(&"\n| | ____ ___  ____     _____| |_____  ____ ____  _____| | ");
	putstr(&"\n| |/ ___) _ \\|  _ \\   |  _   _) ___ |/ ___)  _ \\| ___ | | ");
	putstr(&"\n| | |  | |_| | | | |  | |  \\ \\| ____| |   | | | | ____| | ");
	putstr(&"\n|_|_|  \\____/|_| |_|  |_|   \\_\\_____)_|   |_| |_|_____)__)\n\n");

}

pub unsafe fn init() {
    buffer = strings::cstr::from_str(&"butts");
    screen();
    prompt(true);
}

unsafe fn prompt(startup: bool) {
	putstr(&"\nsgash > ");
	if !startup {drawstr(&"\nsgash > ");}
	buffer.reset();
}

unsafe fn parse() {
	putstr(&"\n");
	putcstr(buffer);
	if (buffer.streq(&"ls")) { 
	    putstr( &"\nWould print directory contents") ;
	    drawstr( &"\nWould print directory contents") ;
	};
	match buffer.getarg(' ', 0) {
	    Some(y)        => {
		if(y.streq(&"cat")) {
		   	putstr(&"\nPrinting file");
		    drawstr(&"\nPrinting file");
		    }
		else if(y.streq(&"wr")) {
		    putstr(&"\nWriting to file");
		    drawstr(&"\nWriting to file");
		}
		else if(y.streq(&"echo")) {
				match buffer.getarg(' ', 1) {
					Some(x) => {
						putstr(&"\n");
						putcstr( x );
						drawstr( &"\nPrinted arg!");
						}
					None => { }
				};
		}
		else if(y.streq(&"rm")) {
			putstr(&"\nRemoving file");
		    drawstr(&"\nRemoving file");
		}
		else if(y.streq(&"cd")) {
			putstr(&"\nChanging directory");
		    drawstr(&"\nChanging directory");
		}
		else if(y.streq(&"mkdir")) {
			putstr(&"\nMaking directory");
		    drawstr(&"\nMaking directory");
		};
	}
	    None        => { }
	};

	if(buffer.streq(&"pwd")) {
		putstr(&"\nPrinting current directory");
		drawstr(&"\nPrinting current directory");
	};

	buffer.reset();
}