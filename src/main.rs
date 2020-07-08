extern crate termion;
extern crate unicode_width;

use std::cmp::min;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{clear, cursor};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
// カーソルの位置　0-indexed
struct Cursor {
    row: usize,
    column: usize,
}

#[derive(Default)]
struct Surara {
    cursor: Cursor,
    vcolumn: usize,
    text: Vec<Vec<char>>,
    change_flag: bool,
}
impl std::fmt::Debug for Surara {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.cursor)?;
        for line in &self.text {
            writeln!(f, "{:?}", line)?;
        }
        Ok(())
    }
}

impl Surara {
    fn new() -> Result<Self, std::io::Error> {
        let args: Vec<String> = env::args().skip(1).collect();
        let mut text: Vec<Vec<char>> = vec![];
        if let Some(filename) = args.first() {
            let reader = BufReader::new(File::open(filename)?);
            for line in reader.lines() {
                text.push(line.unwrap().chars().collect());
            }
        }
        Ok(Self {
            text,
            change_flag: true,
            ..Default::default()
        })
    }
    fn init<T: Write>(&self, stdout: &mut T) -> std::io::Result<()> {
        // 画面全体をクリアする
        write!(stdout, "{}", clear::All)?;
        // カーソルを左上に設定する(1-indexed)
        write!(stdout, "{}", cursor::Goto(1, 1))?;
        self.draw(stdout)?;
        write!(stdout, "{}", cursor::Goto(1, 1))?;
        stdout.flush()?;
        Ok(())
    }
    fn draw<T: Write>(&self, stdout: &mut T) -> std::io::Result<()> {
        if self.change_flag {
            write!(stdout, "{}", clear::All)?;
            write!(stdout, "{}", cursor::Goto(1, 1))?;
            for line in &self.text {
                for &c in line {
                    write!(stdout, "{}", c)?;
                }
                write!(stdout, "\r\n")?;
            }
            stdout.flush()?;
        }
        Ok(())
    }
    fn insert(&mut self, c: char) {
        if c == '\n' {
            // 改行
            let rest: Vec<char> = self.text[self.cursor.row]
                .drain(self.cursor.column..)
                .collect();
            self.text.insert(self.cursor.row + 1, rest);
            self.cursor.row += 1;
            self.cursor.column = 0;
        // self.scroll();
        } else if !c.is_control() {
            self.text[self.cursor.row].insert(self.cursor.column, c);
            self.cursor.column += 1;
        }
        self.change_flag = true
    }
}

fn main() -> std::io::Result<()> {
    let mut app = Surara::new()?;
    let mut stdout = AlternateScreen::from(std::io::stdout().into_raw_mode().unwrap());
    app.init(&mut stdout)?;
    // eventsはTermReadトレイトに定義されている
    let stdin = std::io::stdin();
    for evt in stdin.events() {
        match evt.unwrap() {
            // Ctrl-cでプログラム終了
            // Rawモードなので自前で終了方法を書いてかないと終了する方法がなくなってしまう！
            Event::Key(Key::Ctrl('c')) => {
                break;
            }

            // 方向キーの処理
            Event::Key(Key::Up) => {
                if app.cursor.row > 0 {
                    app.cursor.row -= 1;
                    app.cursor.column = min(app.text[app.cursor.row].len(), app.cursor.column);
                }
            }
            Event::Key(Key::Down) => {
                if app.cursor.row + 1 < app.text.len() {
                    app.cursor.row += 1;
                    app.cursor.column = min(app.cursor.column, app.text[app.cursor.row].len());
                }
            }
            Event::Key(Key::Left) => {
                if app.cursor.column > 0 {
                    app.cursor.column -= 1;
                }
            }
            Event::Key(Key::Right) => {
                app.cursor.column = min(app.cursor.column + 1, app.text[app.cursor.row].len());
            }
            Event::Key(Key::Char(c)) => app.insert(c),
            _ => {}
        }
        app.vcolumn = app.text[app.cursor.row][0..app.cursor.column]
            .iter()
            .map(|c| c.width().unwrap())
            .sum();

        app.draw(&mut stdout)?;
        app.change_flag = false;
        write!(
            stdout,
            "{}",
            cursor::Goto(app.vcolumn as u16 + 1, app.cursor.row as u16 + 1)
        )?;

        stdout.flush().unwrap();
    }
    drop(stdout);
    println!("{:?}", app);
    return Ok(());
}
