use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::Write;

static SYMBOLS: Lazy<Vec<char>> = Lazy::new(|| vec!['=', '!', '<', '>', ':']);
static OPERATORS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut operators = HashMap::new();
    // operators.insert("or", "OR");
    // operators.insert("and", "AND");
    operators.insert("=", "=");
    operators.insert("eq", "=");
    operators.insert("!=", "<>");
    operators.insert("<>", "<>");
    operators.insert("neq", "<>");
    operators.insert("is", "IS");
    operators.insert("nis", "IS NOT");
    operators.insert("in", "IN");
    operators.insert("nin", "NOT IN");
    operators.insert("like", "LIKE");
    operators.insert("nlike", "NOT LIKE");
    operators.insert("ilike", "ILIKE");
    operators.insert("nilike", "NOT ILIKE");
    operators.insert("<", "<");
    operators.insert("lt", "<");
    operators.insert(">", ">");
    operators.insert("gt", "gt");
    operators.insert("<=", "<=");
    operators.insert("lte", "<=");
    operators.insert(">=", ">=");
    operators.insert("gte", ">=");
    operators
});
pub struct Parser {
    pub offset: usize,
    pub raw: String,
    pub chars: Vec<char>,
    pub idents: Vec<String>,
    pub joined_options: Vec<JoinedOption>,
}
#[derive(Clone, Debug)]
pub struct JoinedOption {
    pub outer_table: String,
    pub outer_key: String,
    pub inner_key: String,
    pub url_name_map: HashMap<String, String>,
}
impl Parser {
    pub fn new(raw: String, idents: Vec<String>, options: Vec<JoinedOption>) -> Parser {
        Parser {
            offset: 0,
            chars: raw.chars().collect(),
            raw,
            idents,
            joined_options: options,
        }
    }
    pub fn parse(&mut self) -> Result<String, String> {
        if self.raw.is_empty() {
            return Ok("".into());
        }
        Ok(self.scan_expr()?.trim().to_string())
    }
    fn next(&mut self, skip_blank: bool) -> Option<char> {
        self.offset += 1;
        if self.offset < self.chars.len() {
            if skip_blank {
                self.skip_blank();
            }
            Some(self.chars[self.offset])
        } else {
            None
        }
    }
    fn try_next(&mut self, skip_blank: bool) -> Result<char, String> {
        self.next(skip_blank).ok_or(format!(
            "unexcepted end 1, offset: {}, raw: {}",
            self.offset, &self.raw
        ))
    }
    fn skip_blank(&mut self) {
        let ch = self.curr();
        if ch.is_none() {
            return;
        }
        let mut ch = ch.unwrap();
        while ch == ' ' || ch == '\t' {
            if self.offset < self.chars.len() - 1 {
                self.offset += 1;
                ch = self.chars[self.offset];
            } else {
                break;
            }
        }
    }
    fn peek(&self) -> Option<&char> {
        if self.offset < self.chars.len() - 1 {
            self.chars.get(self.offset + 1)
        } else {
            None
        }
    }
    fn peek_token(&self) -> Option<String> {
        let mut tok = "".to_owned();
        let mut offset = self.offset;
        while offset < self.chars.len() {
            let ch = self.chars[offset];
            if ch != ' ' && ch != '\t' {
                tok.push(ch);
            } else {
                break;
            }
            offset += 1;
        }
        if tok.is_empty() {
            None
        } else {
            // println!("=============toke: {:?}", &tok);
            Some(tok)
        }
    }
    fn curr(&self) -> Option<char> {
        if self.offset < self.chars.len() {
            Some(self.chars[self.offset])
        } else {
            None
        }
    }

    pub fn scan_expr(&mut self) -> Result<String, String> {
        let mut expr: String = "".into();
        // 跳过下面所有的空格
        self.skip_blank();
        if let Some(ch) = self.curr() {
            // 开始一个表达式
            if ch == '(' {
                expr.push('(');
                self.offset += 1;
                // 开始扫描表达式
                expr.push_str(&self.scan_expr()?);
                self.skip_blank();
                if self.curr().unwrap_or_default() == ')' {
                    expr.push(')');
                    self.offset += 1;
                } else {
                    return Err(format!(
                        "excepted ')', offset: {}, raw: {}",
                        self.offset, &self.raw
                    ));
                }
            } else {
                // 已经在表达式内了

                // 扫描表达式左侧， indent
                let left = self.scan_left()?;
                // println!("==============left: {:#?}", &left);

                // 扫描操作符
                let operator = self.scan_operator()?;

                // 扫描值
                let right = self.scan_value()?;
                // println!("==============right: {:#?}", &right);

                // 如果左侧有点， 那就是外键表的字段， 比如book_authors.name
                if left.contains('.') {
                    if let Some(option) = self.find_option(&left) {
                        write!(
                            expr,
                            "{} in (select {} from {} where {} {} {})",
                            option.inner_key,
                            option.outer_key,
                            option.outer_table,
                            option.url_name_map[&left],
                            &operator,
                            right
                        )
                        .unwrap();
                    } else {
                        return Err(format!("ident is not correct, raw: {}", &self.raw));
                    }
                } else {
                    expr.push_str(&left);
                    expr.push(' ');
                    expr.push_str(&operator);
                    expr.push(' ');
                    expr.push_str(&right);
                }
            }
            self.skip_blank();

            // 找到下一个token?

            if let Some(tok) = self.peek_token() {
                // 一般在search里会有， 连接词， 连接下一个子句
                if tok == "and" || tok == "or" {
                    expr.push(' ');
                    expr.push_str(&tok);
                    expr.push(' ');
                    if tok == "and" {
                        self.offset += 4;
                    } else {
                        self.offset += 3;
                    }

                    // 继续扫描下一个表达式
                    expr.push_str(&self.scan_expr()?);
                }
            }
        }
        Ok(expr)
    }

    fn find_option(&self, url_name: &str) -> Option<&JoinedOption> {
        self.joined_options
            .iter()
            .find(|&option| option.url_name_map.get(url_name).is_some())
    }

    fn scan_left(&mut self) -> Result<String, String> {
        self.skip_blank();
        let offset = self.offset;
        match self.scan_ident() {
            Ok(ident) => Ok(ident),
            Err(_) => {
                self.offset = offset;
                self.scan_value()
            }
        }
    }

    // 扫描字段名
    fn scan_ident(&mut self) -> Result<String, String> {
        self.skip_blank();
        let mut ident = "".to_owned();
        let mut ch = self.curr().unwrap();
        // 一直扫，直到遇到空格或者操作符
        while !ch.is_whitespace() && !SYMBOLS.contains(&ch) {
            ident.push(ch);
            if let Some(c) = self.next(false) {
                ch = c;
            } else {
                break;
            }
        }

        if ident.contains('.') {
            // 有点， 那基本上是外键表的字段， 比如book_authors.name
            match self.find_option(&ident) {
                Some(_) => Ok(ident),
                None => Err(format!(
                    "ident is not allowed 0, offset:{}, raw: {}",
                    self.offset, &self.raw
                )),
            }
        } else if self.idents.contains(&ident) {
            // 如果后面是冒号， 那就是id::varchar(255)这种, 应该紧跟两个冒号
            if let Some(':') = self.curr() {
                if let Some(':') = self.peek() {
                    // 向后走
                    self.next(false);
                    // 把两个冒号推进来
                    ident.push_str("::");

                    let mut ch = self.try_next(false)?;

                    // 一般就是收集varchar(255)这种
                    while !ch.is_whitespace() && !SYMBOLS.contains(&ch) {
                        ident.push(ch);
                        if let Some(c) = self.next(false) {
                            ch = c;
                        } else {
                            break;
                        }
                    }
                } else {
                    // 如果不是两个冒号， 报错
                    return Err(format!(
                        "':' is not allowed here, offset:{}, raw: {}, idents: {:#?}, joined_options: {:#?}, ident: {}",
                        self.offset, &self.raw, &self.idents, &self.joined_options, &ident
                    ));
                }
            }
            Ok(ident)
        } else {
            // 如果不在字段列表里，报错
            Err(format!(
                "ident is not allowed 1, offset:{}, raw: {}, idents: {:#?}, joined_options: {:#?}, ident: {}",
                self.offset, &self.raw, &self.idents, &self.joined_options, &ident
            ))
        }
    }
    fn scan_value(&mut self) -> Result<String, String> {
        self.skip_blank();
        let mut value = "".to_owned();
        let mut ch = self.curr().unwrap();
        if ch == '\'' || ch == 'E' {
            if ch == 'E' {
                value.push(ch);
                ch = self.try_next(false)?;
                if ch != '\'' {
                    return Err("except ' after E".to_owned());
                }
            }
            value.push(ch);
            ch = self.try_next(false)?;
            loop {
                value.push(ch);
                if ch == '\\' {
                    value.push(self.try_next(false)?);
                    ch = self.try_next(false)?;
                } else if ch == '\'' {
                    if let Some('\'') = self.peek() {
                        value.push(self.try_next(false)?);
                        ch = self.try_next(false)?;
                    } else {
                        self.next(false);
                        break;
                    }
                } else {
                    ch = self.try_next(false)?;
                }
            }
        } else {
            let mut brackets = 0;
            while !ch.is_whitespace() && ch != '=' {
                if ch == ')' {
                    if brackets == 0 {
                        break;
                    }
                    brackets -= 1;
                }
                if ch == '(' {
                    brackets += 1;
                }
                value.push(ch);
                if let Some(c) = self.next(false) {
                    ch = c;
                } else {
                    break;
                }
            }
        }
        Ok(value)
    }

    fn scan_operator(&mut self) -> Result<String, String> {
        self.skip_blank();
        let mut url_opt = "".to_owned();
        let mut ch = self.curr().unwrap();
        let is_symbol = SYMBOLS.contains(&ch);
        if is_symbol {
            while SYMBOLS.contains(&ch) {
                url_opt.push(ch);
                if let Some(c) = self.next(false) {
                    ch = c;
                } else {
                    break;
                }
            }
        } else {
            while !ch.is_whitespace() {
                url_opt.push(ch);
                if let Some(c) = self.next(false) {
                    ch = c;
                } else {
                    break;
                }
            }
        }
        OPERATORS
            .get(&&*url_opt)
            .cloned()
            .map(|s| s.to_owned())
            .ok_or(format!(
                "operator is not correct, raw: {}, url_opt:{}, operators: {:#?}",
                &self.raw, &url_opt, &*OPERATORS
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let raw = "(id::varchar(255)='京极夏彦' or title ilike E'%京极夏彦%' or exists (select 1 from book_authors where book_authors.name ilike E'%京极夏彦%' and book_authors.id = any(author))".to_owned();
        let idents = vec!["id".to_owned(), "title".to_owned(), "author".to_owned()];
        let options = vec![];
        let mut parser = Parser::new(raw, idents, options);
        let result = parser.parse();
        println!("result: {:?}", &result);
        assert!(result.is_ok());
    }
    #[test]
    fn test_parse2() {
        let raw = "id::varchar(255)='京极夏彦' or name ilike E'%京极夏彦%'".to_owned();
        let idents = vec!["id".to_owned(), "title".to_owned(), "author".to_owned()];
        let options = vec![];
        let mut parser = Parser::new(raw, idents, options);
        let result = parser.parse();
        println!("result: {:?}", &result);
        assert!(result.is_ok());
    }
}
