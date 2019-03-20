pipeline {
    agent none
    stages {
        stage('Check Formatting and Clippy') {
            environment {
                CARGO_HOME = '/home/jenkins/.cargo'
                RUSTUP_HOME = '/home/jenkins/.rustup'
                RUSTFLAGS = "-D warnings"
            }
            agent {
                label 'linux'
            }
            steps {
                echo 'Checking formatting...'
                sh '$CARGO_HOME/bin/cargo fmt -- --check'
            }
        }
        stage('Run Clippy') {
            environment {
                CARGO_HOME = '/home/jenkins/.cargo'
                RUSTUP_HOME = '/home/jenkins/.rustup'
                RUSTFLAGS = "-D warnings"
            }
            agent {
                label 'linux'
            }
            steps {
                echo 'Running Clippy...'
                sh '$CARGO_HOME/bin/cargo clippy --all --all-features -- -D warnings'
            }
        }
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
                        echo 'Beginning tests...'
                        bat 'C:\\Users\\root\\.cargo\\bin\\cargo +stable test'
                        echo 'Tests done!'
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
                        echo 'Beginning tests...'
                        sh '/home/jenkins/.cargo/bin/cargo test'
                        echo 'Tests done!'
                    }
                }
                stage("Test on macOS") {
                    environment {
                        CARGO_HOME = '/Users/jenkins/.cargo'
                        RUSTUP_HOME = '/Users/jenkins/.rustup'
                    }
                    agent {
                        label 'mac'
                    }
                    steps {
                        echo 'Beginning tests...'
                        sh '/Users/jenkins/.cargo/bin/cargo test'
                        echo 'Tests done!'
                    }
                }
            }
        }
    }
}
