use regex::Regex;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser};

#[derive(Clone)]
pub struct AnalyzedFile {
    pub file_tokens: Vec<Token>,
    pub functions: HashMap<String, Vec<Token>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Var,
    Func,
    ConstInt,
    ConstFloat,
    ConstStr,
    Loop,
    If,
    ArithOp,
    LogicOp,
    RelOp,
    Other(String),
}

impl Token {
    fn as_str(&self) -> &str {
        match self {
            Token::Var => "Var",
            Token::Func => "Func",
            Token::ConstInt => "ConstInt",
            Token::ConstFloat => "ConstFloat",
            Token::ConstStr => "ConstStr",
            Token::Loop => "Loop",
            Token::If => "If",
            Token::ArithOp => "ArithOp",
            Token::LogicOp => "LogicOp",
            Token::RelOp => "RelOp",
            Token::Other(_) => "Other",
        }
    }
}

fn remove_comments(code: &str) -> String {
    let block_re = Regex::new(r"/\*.*?\*/").unwrap();
    let line_re = Regex::new(r"//.*").unwrap();
    let code = block_re.replace_all(code, "");
    let code = line_re.replace_all(&code, "");
    code.to_string()
}

fn remove_hashtags(code: &str) -> String {
    code.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn preprocess_code(code: &str) -> String {
    let code = remove_comments(code);
    let code = remove_hashtags(&code);
    let code = Regex::new(r"typedef .*?;").unwrap().replace_all(&code, "");
    let code = Regex::new(r"#define .*").unwrap().replace_all(&code, "");
    code.to_string()
}

fn extract_tokens(node: Node, source: &str, tokens: &mut Vec<Token>) {
    match node.kind() {
        "identifier" => tokens.push(Token::Var),
        "function_definition" | "call_expression" => tokens.push(Token::Func),
        "integer_literal" => tokens.push(Token::ConstInt),
        "float_literal" => tokens.push(Token::ConstFloat),
        "string_literal" => tokens.push(Token::ConstStr),
        "for_statement" | "while_statement" | "do_statement" => tokens.push(Token::Loop),
        "if_statement" => tokens.push(Token::If),
        "binary_expression" => {
            if let Some(op_node) = node.child_by_field_name("operator") {
                let op = op_node.utf8_text(source.as_bytes()).unwrap();
                match op {
                    "+" | "-" | "*" | "/" | "%" => tokens.push(Token::ArithOp),
                    "&&" | "||" | "!" => tokens.push(Token::LogicOp),
                    "<" | ">" | "<=" | ">=" | "==" | "!=" => tokens.push(Token::RelOp),
                    _ => tokens.push(Token::Other(op.to_string())),
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            extract_tokens(child, source, tokens);
        }
    }
}

fn extract_functions(node: Node, source: &str) -> HashMap<String, Vec<Token>> {
    let mut functions = HashMap::new();

    if node.kind() == "function_definition" {
        if let Some(declarator) = node.child_by_field_name("declarator") {
            if let Some(name_node) = declarator
                .child_by_field_name("identifier")
                .or_else(|| declarator.named_child(0))
            {
                let func_name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("<unnamed>");

                let mut tokens = Vec::new();
                extract_tokens(node, source, &mut tokens);
                functions.insert(func_name.to_string(), tokens);
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            let nested = extract_functions(child, source);
            functions.extend(nested);
        }
    }

    functions
}

fn jaccard_similarity(seq1: &[Token], seq2: &[Token]) -> f32 {
    let set1: HashSet<_> = seq1.iter().collect();
    let set2: HashSet<_> = seq2.iter().collect();

    if set1.is_empty() && set2.is_empty() {
        return 1.0;
    }

    let intersection: HashSet<_> = set1.intersection(&set2).collect();
    let union: HashSet<_> = set1.union(&set2).collect();

    intersection.len() as f32 / union.len() as f32
}

fn lcs_similarity<T: PartialEq>(a: &[T], b: &[T]) -> f32 {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }

    let lcs_len = dp[m][n];
    lcs_len as f32 / ((m + n) as f32 / 2.0)
}

fn token_weight(token: &Token) -> f32 {
    match token {
        Token::If => 2.0,
        Token::Loop => 2.0,
        Token::Func => 1.5,
        Token::Var => 0.5,
        Token::ConstInt => 0.5,
        Token::ConstFloat => 0.5,
        Token::ConstStr => 0.5,
        _ => 1.0,
    }
}

fn weighted_cosine(tokens1: &[Token], tokens2: &[Token]) -> f32 {
    let all_tokens: HashSet<_> = tokens1.iter().chain(tokens2.iter()).collect();

    let mut w1 = HashMap::new();
    let mut w2 = HashMap::new();

    for token in all_tokens.iter() {
        let count1 = tokens1.iter().filter(|t| *t == *token).count() as f32;
        let count2 = tokens2.iter().filter(|t| *t == *token).count() as f32;
        w1.insert(*token, count1 * token_weight(token));
        w2.insert(*token, count2 * token_weight(token));
    }

    let mut dot = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;

    for token in all_tokens {
        let v1 = *w1.get(token).unwrap_or(&0.0);
        let v2 = *w2.get(token).unwrap_or(&0.0);
        dot += v1 * v2;
        norm1 += v1 * v1;
        norm2 += v2 * v2;
    }

    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        dot / (norm1.sqrt() * norm2.sqrt())
    }
}

fn combined_similarity(tokens1: &[Token], tokens2: &[Token]) -> f32 {
    let seq1_string: String = tokens1
        .iter()
        .map(|t| t.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let seq2_string: String = tokens2
        .iter()
        .map(|t| t.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    let sm = strsim::normalized_levenshtein(&seq1_string, &seq2_string) as f32;

    let jacc = jaccard_similarity(tokens1, tokens2);
    let lcs = lcs_similarity(tokens1, tokens2);
    let cos = weighted_cosine(tokens1, tokens2);

    0.4 * sm + 0.2 * jacc + 0.2 * lcs + 0.2 * cos
}

fn similaridade_funcoes(
    funcoes1: &HashMap<String, Vec<Token>>,
    funcoes2: &HashMap<String, Vec<Token>>,
) -> f32 {
    if funcoes1.is_empty() || funcoes2.is_empty() {
        return 0.0;
    }

    let mut resultados = Vec::new();

    for f1_tokens in funcoes1.values() {
        for f2_tokens in funcoes2.values() {
            let sim = combined_similarity(f1_tokens, f2_tokens);
            resultados.push(sim);
        }
    }

    *resultados
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(&0.0)
}

fn parse_code(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .expect("Error loading C grammar");
    parser.parse(source, None).unwrap()
}

pub fn compare_two_codes(code1: &str, code2: &str) -> f32 {
    if code1.trim().is_empty() || code2.trim().is_empty() {
        return -1.0;
    }

    let mut questoes: Vec<(Vec<Token>, HashMap<String, Vec<Token>>)> = Vec::new();

    for code in &[code1, code2] {
        let code_clean = preprocess_code(code);

        let tree = parse_code(&code_clean);

        let mut file_tokens = Vec::new();
        extract_tokens(tree.root_node(), &code_clean, &mut file_tokens);

        let functions = extract_functions(tree.root_node(), &code_clean);

        questoes.push((file_tokens, functions));
    }

    let sim_arquivo = combined_similarity(&questoes[0].0, &questoes[1].0);
    let sim_funcoes = similaridade_funcoes(&questoes[0].1, &questoes[1].1);

    (sim_arquivo + sim_funcoes) / 2.0
}

pub fn compare_two_codes_cached(analyzed1: &AnalyzedFile, analyzed2: &AnalyzedFile) -> f32 {
    let sim_arquivo = combined_similarity(&analyzed1.file_tokens, &analyzed2.file_tokens);
    let sim_funcoes = similaridade_funcoes(&analyzed1.functions, &analyzed2.functions);

    (sim_arquivo + sim_funcoes) / 2.0
}

pub fn analyze_code(code: &str) -> Option<AnalyzedFile> {
    if code.trim().is_empty() {
        return None;
    }

    let code_clean = preprocess_code(code);
    let tree = parse_code(&code_clean);

    let mut file_tokens = Vec::new();
    extract_tokens(tree.root_node(), &code_clean, &mut file_tokens);

    let functions = extract_functions(tree.root_node(), &code_clean);

    Some(AnalyzedFile {
        file_tokens,
        functions,
    })
}
