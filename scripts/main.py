import os
import difflib
import itertools
from pycparser import c_parser, c_ast
import re
from collections import defaultdict
import json
import os

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

        # Normalização
        if token == "ID":
            self.tokens.append("VAR")
        elif token == "FuncCall":
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
            self.tokens.append(node.op)
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
# Similaridade combinada
# -------------------------------


def jaccard_similarity(seq1, seq2):
    set1, set2 = set(seq1), set(seq2)
    if not set1 and not set2:
        return 1.0
    return len(set1 & set2) / len(set1 | set2)


def similaridade_combinada(seq1, seq2, peso_seq=0.8, peso_jaccard=0.2):
    sm = difflib.SequenceMatcher(None, seq1, seq2).ratio()
    jacc = jaccard_similarity(seq1, seq2)
    return peso_seq * sm + peso_jaccard * jacc


def similaridade_funcoes(funcoes1, funcoes2):
    """Compara função a função e retorna a maior similaridade combinada"""
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


# -------------------------------
# Carregar arquivos
# -------------------------------
def carregar_questoes(pasta_base, alunos_json):
    questoes = {}  # { "q1_alunoX": {"assinatura":..., "funcoes":...} }

    for aluno in os.listdir(pasta_base):
        caminho_aluno = os.path.join(pasta_base, aluno)
        if not os.path.isdir(caminho_aluno):
            continue

        aluno_nome = caminho_aluno.split("/")[-1]
        alunos_json[aluno_nome] = {"erros": []}

        for arquivo in os.listdir(caminho_aluno):
            if arquivo.endswith(".c"):
                caminho_arquivo = os.path.join(caminho_aluno, arquivo)

                try:
                    with open(caminho_arquivo, "r", encoding="utf-8") as f:
                        codigo = f.read()
                except:
                    print(f"Não foi possível abrir o arquivo: {caminho_aluno}")
                    alunos_json[aluno_nome]["erros"].append(
                        f"ERRO AO ABRIR ARQUIVO {arquivo}")
                    continue

                if not codigo.strip():
                    print(f"Aviso: arquivo vazio {arquivo} ignorado")
                    alunos_json[aluno_nome]["erros"].append(
                        f"[001] ARQUIVO {arquivo} VAZIO")
                    continue

                codigo_limpo = remover_hashtag(codigo)
                codigo_limpo = remover_comentarios(codigo_limpo)

                ast = gerar_ast(codigo_limpo)
                if not ast:
                    print(
                        f"Aviso: arquivo {
                            arquivo} não pôde ser parseado e será ignorado"
                    )
                    alunos_json[aluno_nome]["erros"].append(
                        f"[002] ARQUIVO {arquivo} NÃO PÔDE SER PARSEADO")
                    continue

                assinatura = extrair_assinatura(ast)

                func_visitor = ASTFunctionVisitor()
                func_visitor.visit(ast)

                chave = arquivo.split(".c")[0]  # q1_nomealuno
                questoes[chave] = {
                    "assinatura": assinatura,
                    "funcoes": func_visitor.functions,
                }

    return questoes


# -------------------------------
# Comparar arquivos da mesma questão
# -------------------------------
def comparar_questoes(questoes, alunos_json, threshold=0.85):
    resultados = []

    # Agrupa arquivos por questão
    grupos = defaultdict(list)
    for chave in questoes.keys():
        prefixo = chave.split("_")[0]  # q1, q2, ...
        grupos[prefixo].append(chave)

    # Compara arquivos dentro do mesmo grupo
    for prefixo, arquivos in grupos.items():
        for a, b in itertools.combinations(arquivos, 2):
            assinatura_a = questoes[a]["assinatura"]
            assinatura_b = questoes[b]["assinatura"]
            funcoes_a = questoes[a]["funcoes"]
            funcoes_b = questoes[b]["funcoes"]

            sim_arquivo = similaridade_combinada(assinatura_a, assinatura_b)
            sim_funcoes = similaridade_funcoes(funcoes_a, funcoes_b)

            sim_final = max(sim_arquivo, sim_funcoes)

            if sim_final >= threshold:
                resultados.append((a, b, sim_final))
                aluno_a_nome = a.split("_")[1]
                alunos_json[aluno_a_nome]["erros"].append(
                    f"[003] COPIA DETECTADA ENTRE {a} e {
                        b} DE {sim_final * 100:.2f}"
                )

    resultados.sort(key=lambda x: x[2], reverse=True)
    return resultados


# -------------------------------
# Relatório HTML
# -------------------------------
def gerar_relatorio_html(resultados, arquivo_saida="relatorio.html"):
    html = "<html><head><style>"
    html += "table {border-collapse: collapse; width: 80%;}"
    html += "th, td {border: 1px solid black; padding: 5px; text-align: left;}"
    html += ".alto {background-color: #ff9999;}"  # vermelho claro
    html += ".medio {background-color: #ffcc99;}"  # laranja
    html += ".baixo {background-color: #ffff99;}"  # amarelo
    html += "</style></head><body>"
    html += "<h2>Relatório de Similaridade entre Questões</h2>"
    html += (
        "<table><tr><th>Arquivo A</th><th>Arquivo B</th><th>Similaridade (%)</th></tr>"
    )

    for a, b, sim in resultados:
        if sim > 0.9:
            classe = "alto"
        elif sim > 0.85:
            classe = "medio"
        else:
            classe = "baixo"
        html += (
            f"<tr class='{classe}'><td>{
                a}</td><td>{b}</td><td>{sim*100:.2f}</td></tr>"
        )

    html += "</table></body></html>"

    with open(arquivo_saida, "w", encoding="utf-8") as f:
        f.write(html)

    print(f"Relatório HTML gerado: {arquivo_saida}")


# -------------------------------
# Uso
# -------------------------------


def main(path_alunos, json_output, dificuldade):
    alunos_json = {}
    path_alunos
    questoes = carregar_questoes(path_alunos, alunos_json)

    threshold = 1 - (dificuldade - 1) / 20

    resultados = comparar_questoes(questoes, alunos_json, threshold)

    # gerar_relatorio_html(resultados)

    os.makedirs("resultados", exist_ok=True)

    with open(json_output, "w", encoding="utf-8") as f:
        json.dump(alunos_json, f, ensure_ascii=False, indent=4)

    return alunos_json


if __name__ == "__main__":
    pasta_base = "../submissions/790450373123/790450373216/"
    json_output = "resultados/data.json"

    resultados_json = main(pasta_base, json_output, 2)

    # print("\n📊 Similaridade entre questões dos alunos (pares suspeitos >85%):\n")
    # for a, b, sim in resultados:
    #     print(f"{a}  <-->  {b}  =>  {sim*100:.2f}%")
