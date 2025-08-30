import difflib
import itertools
from pycparser import c_parser, c_ast
import re
import os
import math
from collections import Counter

# -------------------------------
# Funções AST
# -------------------------------


def gerar_ast(code: str):
    parser = c_parser.CParser()
    try:
        return parser.parse(code)
    except Exception as e:
        print("Erro ao parsear código:", e)
        return None


class ASTSignature(c_ast.NodeVisitor):
    """Cria uma assinatura normalizada da AST"""

    def __init__(self):
        self.tokens = []

    def generic_visit(self, node):
        token = type(node).__name__

        # Normalização mais forte
        if token == "ID":
            self.tokens.append("VAR")
        elif token == "FuncCall":
            self.tokens.append("FUNC")
        elif token == "FuncDef":
            self.tokens.append("FUNC")
        elif token == "Constant":
            if node.type == "int":
                self.tokens.append("CONST_INT")
            elif node.type == "float":
                self.tokens.append("CONST_FLOAT")
            elif node.type == "char":
                self.tokens.append("CONST_CHAR")
            elif node.type == "string":
                self.tokens.append("CONST_STR")
            else:
                self.tokens.append("CONST")
        elif token in ["BinaryOp", "UnaryOp"]:
            if node.op in ["+", "-", "*", "/", "%"]:
                self.tokens.append("ARITH_OP")
            elif node.op in ["&&", "||", "!"]:
                self.tokens.append("LOGIC_OP")
            elif node.op in ["<", ">", "<=", ">=", "==", "!="]:
                self.tokens.append("REL_OP")
            else:
                self.tokens.append(node.op)
        elif token in ["For", "While", "DoWhile"]:
            self.tokens.append("LOOP")
        else:
            self.tokens.append(token)

        super().generic_visit(node)


def extrair_assinatura(ast):
    visitor = ASTSignature()
    visitor.visit(ast)
    return visitor.tokens


# -------------------------------
# Comparação por função
# -------------------------------
class ASTFunctionVisitor(c_ast.NodeVisitor):
    """Extrai assinaturas de cada função"""

    def __init__(self):
        self.functions = {}  # {nome_func: assinatura_tokens}

    def visit_FuncDef(self, node):
        sig_visitor = ASTSignature()
        sig_visitor.visit(node)
        nome_func = getattr(node.decl, "name", "FUNC")
        self.functions[nome_func] = sig_visitor.tokens
        self.generic_visit(node)


# -------------------------------
# Similaridade
# -------------------------------

def jaccard_similarity(seq1, seq2):
    set1, set2 = set(seq1), set(seq2)
    if not set1 and not set2:
        return 1.0
    return len(set1 & set2) / len(set1 | set2)


def lcs_length(seq1, seq2):
    """Longest Common Subsequence"""
    m, n = len(seq1), len(seq2)
    dp = [[0] * (n + 1) for _ in range(m + 1)]
    for i in range(m):
        for j in range(n):
            if seq1[i] == seq2[j]:
                dp[i + 1][j + 1] = dp[i][j] + 1
            else:
                dp[i + 1][j + 1] = max(dp[i][j + 1], dp[i + 1][j])
    return dp[m][n] / max(1, min(m, n))


TOKEN_PESOS = {
    "IF": 2.0,
    "LOOP": 2.0,
    "FUNC": 1.5,
    "VAR": 0.5,
    "CONST_INT": 0.5,
    "CONST_STR": 0.5,
}


def weighted_cosine_similarity(seq1, seq2):
    all_tokens = set(seq1) | set(seq2)
    w1 = {t: seq1.count(t) * TOKEN_PESOS.get(t, 1.0) for t in all_tokens}
    w2 = {t: seq2.count(t) * TOKEN_PESOS.get(t, 1.0) for t in all_tokens}
    dot = sum(w1[t] * w2[t] for t in all_tokens)
    norm1 = math.sqrt(sum(v * v for v in w1.values()))
    norm2 = math.sqrt(sum(v * v for v in w2.values()))
    return dot / (norm1 * norm2 + 1e-9)


def similaridade_combinada(seq1, seq2):
    sm = difflib.SequenceMatcher(None, seq1, seq2).ratio()
    jacc = jaccard_similarity(seq1, seq2)
    lcs = lcs_length(seq1, seq2)
    cos = weighted_cosine_similarity(seq1, seq2)
    return 0.4 * sm + 0.2 * jacc + 0.2 * lcs + 0.2 * cos


def similaridade_funcoes(funcoes1, funcoes2):
    if not funcoes1 or not funcoes2:
        return 0.0
    resultados = []
    for f1, f2 in itertools.product(funcoes1.keys(), funcoes2.keys()):
        sim = similaridade_combinada(funcoes1[f1], funcoes2[f2])
        resultados.append(sim)
    return max(resultados)


# -------------------------------
# Utilitárias
# -------------------------------

def remover_comentarios(codigo):
    codigo = re.sub(r"/\*.*?\*/", "", codigo, flags=re.DOTALL)
    codigo = re.sub(r"//.*", "", codigo)
    return codigo


def remover_hashtag(codigo):
    return "\n".join(
        linha for linha in codigo.splitlines() if not linha.strip().startswith("#")
    )


def preprocess_code(codigo):
    codigo = remover_hashtag(codigo)
    codigo = remover_comentarios(codigo)
    codigo = re.sub(r"typedef .*?;", "", codigo)
    codigo = re.sub(r"#define .*", "", codigo)
    return codigo.strip()


# -------------------------------
# Comparar arquivos
# -------------------------------

def comparar_dois_codigos(path_codigo_a, path_codigo_b):
    questoes = []

    for arquivo in (path_codigo_a, path_codigo_b):
        if arquivo.endswith(".c"):
            try:
                with open(arquivo, "r", encoding="utf-8") as f:
                    codigo = f.read()
            except:
                print(f"Não foi possível abrir o arquivo: {arquivo}")
                return

            if not codigo.strip():
                print(f"Aviso: arquivo vazio {arquivo} ignorado")
                return

            codigo_limpo = preprocess_code(codigo)
            ast = gerar_ast(codigo_limpo)
            if not ast:
                return

            assinatura = extrair_assinatura(ast)
            func_visitor = ASTFunctionVisitor()
            func_visitor.visit(ast)

            questoes.append((assinatura, func_visitor.functions))

    sim_arquivo = similaridade_combinada(questoes[0][0], questoes[1][0])
    sim_funcoes = similaridade_funcoes(questoes[0][1], questoes[1][1])

    return (sim_arquivo + sim_funcoes) / 2


# -------------------------------
# Uso
# -------------------------------


if __name__ == "__main__":
    print(
        comparar_dois_codigos(
            "../submissions/790450373123/790450373216/abscl@cesar.school/q3_abscl@cesar.school.c",
            "../submissions/790450373123/790450373216/bgnp@cesar.school/q3_bgnp@cesar.school.c",
        )
    )