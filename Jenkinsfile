pipeline {
    environment {
        CARGO_HOME = 'C:\\Users\\root\\.cargo'
        RUSTUP_HOME = 'C:\\Users\\root\\.rustup'
    }
    agent none
    stages {
        stage('Test on Windows') {
            agent { 
                label 'windows' 
            }
            steps {
                bat 'C:\\Users\\root\\.cargo\\bin\\cargo +stable test'
            }
        }
    }
}
