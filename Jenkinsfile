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
                        CARGO_HOME = '/home/jenkins/.cargo'
                        RUSTUP_HOME = '/home/jenkins/.rustup'
                    }
                    agent {
                        label 'linux'
                    }
                    steps {
                        sh '/home/jenkins/.cargo/bin/cargo test'
                    }
                }
                stage("Test on macOS") {
                    environment {
                        CARGO_HOME = '/home/jenkins/.cargo'
                        RUSTUP_HOME = '/home/jenkins/.rustup'
                    }
                    agent {
                        label 'mac'
                    }
                    steps {
                        sh 'cargo test'
                    }
                }
            }
        }
    }
}
