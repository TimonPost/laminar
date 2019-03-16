pipeline {
    agent none
    stages {
        stage('Run Tests') {
            parallel {
                stage("Test on Windows") {                    
                    environment {
                        CARGO_HOME = 'C:\\Users\\root\\.cargo'
                        RUSTUP_HOME = 'C:\\Users\\root\\.rustup'
                    }
                    agent { 
                        label 'windows' 
                    }
                    steps {
                        bat 'C:\\Users\\root\\.cargo\\bin\\cargo +stable test'
                    }
                }
                stage("Test on Linux") {
                    environment {
                        CARGO_HOME = '/root/.cargo'
                        RUSTUP_HOME = '/root/.rustup'
                    }
                    agent {
                        label 'linux'
                    }
                    steps {
                        sh 'sudo /root/.cargo/bin/cargo test'   
                    }
                }
            }
        }
    }
}