use unicode_width::UnicodeWidthStr;

use std::fmt::Write;

#[derive(Debug)]
pub struct TabstopsLines {
    pub lines: Vec<TabstopsLine>,
}

#[derive(Debug)]
pub struct Group {
    pub depth: usize,
    pub start: usize,
    pub end: usize,
    pub width: usize,
}

impl TabstopsLines {
    pub fn new(source: String) -> TabstopsLines {
        let lines: Vec<TabstopsLine> = source
            .lines()
            .map(|line| {
                let block_strs: Vec<String> =
                    line.split("\t").map(|block| block.to_string()).collect();
                let mut blocks = Vec::new();
                for i in 0..block_strs.len() {
                    let block_str = block_strs.get(i).unwrap();
                    let has_next = i != block_strs.len() - 1;
                    blocks.push(TabstopsBlock {
                        adjust_width: 0,
                        has_next: has_next,
                        width: if has_next {
                            block_str.width_cjk() + 1
                        } else {
                            block_str.width_cjk()
                        },
                        block_string: block_str.to_string(),
                    })
                }
                TabstopsLine { blocks: blocks }
            })
            .collect();
        let mut tabstops_lines = TabstopsLines { lines: lines };

        let mut groups = Vec::new();
        for i in 0..tabstops_lines.max_depth() {
            groups.append(&mut tabstops_lines.groups(i));
        }
        tabstops_lines.update_width(groups);
        tabstops_lines
    }

    fn max_depth(&self) -> usize {
        self.lines
            .iter()
            .map(|line| line.blocks.len())
            .max()
            .unwrap()
    }

    fn groups(&self, depth: usize) -> Vec<Group> {
        let mut group_tuples = Vec::new();

        let mut start: Option<usize> = Option::None;
        let mut end: Option<usize> = Option::None;
        let mut current_max_width: usize = 4;

        for (i, line) in self.lines.iter().enumerate() {
            let tab_break_line = match line.blocks.get(depth) {
                Option::None => true,
                Option::Some(block) => {
                    if block.has_next && current_max_width < block.width {
                        current_max_width = block.width;
                    }
                    block.block_string == ""
                }
            };
            if tab_break_line {
                start.map(|start| {
                    end.map(|end| {
                        let group = Group {
                            depth: depth,
                            start: start,
                            end: end,
                            width: current_max_width,
                        };
                        group_tuples.push(group);
                    })
                });
                start = Option::None;
                end = Option::None;
                current_max_width = 4;
            }
            if let Option::None = start {
                start = Option::Some(i);
            }
            end = Option::Some(i);
        }
        start.map(|start| {
            end.map(|end| {
                let group = Group {
                    depth: depth,
                    start: start,
                    end: end,
                    width: current_max_width,
                };
                group_tuples.push(group);
            })
        });

        group_tuples
    }

    fn update_width(&mut self, groups: Vec<Group>) {
        for group in groups {
            for line_index in group.start..=group.end {
                self.lines
                    .get_mut(line_index)
                    .unwrap()
                    .set_adjust_width(group.depth, group.width)
            }
        }
    }

    pub fn to_string(self) -> String {
        let mut result = String::new();
        for line in self.lines {
            for block in line.blocks {
                write!(
                    result,
                    "{space:<indent$}",
                    space = block.block_string,
                    indent = if block.has_next {
                        block.adjust_width
                    } else {
                        block.width
                    }
                )
                .unwrap();
            }
            writeln!(result).unwrap();
        }
        result
    }
}

#[derive(Debug)]
pub struct TabstopsLine {
    pub blocks: Vec<TabstopsBlock>,
}

impl TabstopsLine {
    fn set_adjust_width(&mut self, block_index: usize, adjust_width: usize) {
        self.blocks
            .get_mut(block_index)
            .map(|block| block.adjust_width = adjust_width);
    }
}

#[derive(Debug)]
pub struct TabstopsBlock {
    pub adjust_width: usize,
    pub width: usize,
    pub has_next: bool,
    pub block_string: String,
}

#[cfg(test)]
mod tests {
    use crate::tabstops::TabstopsLines;

    #[test]
    fn test_simple() {
        assert(
            "\
var hoge\t= 123;
var mogegegegege\t= 234;
var a\t= 345;
",
            "\
var hoge         = 123;
var mogegegegege = 234;
var a            = 345;
",
        );
    }

    #[test]
    fn test_tsv() {
        assert(
            "\
positive\tinterest\tleaving\tbat\tgolden\tfeel
news\tfinest\tearth\tbut\tpeace\twall
hard\tmountain\tcheese\tpupil\trailroad\twhistle
largest\tlength\trefer\talso\tletter\ttaken
easily\tjet\tyoung\talready\tsoap\tgulf
fast\tdirt\tbasis\thow\tlibrary\tflame
",
            "\
positive interest leaving bat     golden   feel
news     finest   earth   but     peace    wall
hard     mountain cheese  pupil   railroad whistle
largest  length   refer   also    letter   taken
easily   jet      young   already soap     gulf
fast     dirt     basis   how     library  flame
",
        );
    }

    #[test]
    fn test_source() {
        assert(
            "\
function hoge() {
\tvar x = 0;\t/* comment1 */
\tvar xxxyyyzzz = 2;\t/* comment2 */
}
",
            "\
function hoge() {
    var x = 0;         /* comment1 */
    var xxxyyyzzz = 2; /* comment2 */
}
",
        );
    }

    fn assert(input: &str, expect: &str) {
        assert_eq!(
            TabstopsLines::new(String::from(input)).to_string(),
            String::from(expect)
        );
    }
}
