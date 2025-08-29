use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "subgrader",
    about = "Uma ferramenta para correção de exercícios do Google Classroom"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Comandos relacionados ao Google Classroom
    Classroom {
        #[command(subcommand)]
        command: ClassroomCommands,
    },

    /// Comandos relacionados ao comparador de similaridade
    Similarity {
        #[command(subcommand)]
        command: SimilarityCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClassroomCommands {
    /// Menu interativo para o download das submissões de um exercício
    Assistant,

    /// Lista os cursos em que o usuário faz parte
    ListCourses,

    /// Lista os exercícios dentro de um curso
    ListAssignments {
        #[arg(short, long)]
        course_id: String,
    },

    /// Faz o download de todas as submissões de um exercício
    DownloadSubmissions {
        #[arg(short, long)]
        course_id: String,

        #[arg(short, long)]
        assignment_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SimilarityCommands {
    // Compara a similaridade de 2 arquivos
    Compare {
        #[arg(short, long)]
        file1: String,

        #[arg(short, long)]
        file2: String,
    },
}
