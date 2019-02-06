pipeline {
    agent any
    stages {
        stage('fmt') {
            steps {
                sh 'rustup component add clippy'
                sh 'rustup component add rustfmt'
                sh 'rustup install stable'                
                sh 'cargo fmt --all -- --check'
                sh 'cargo clippy --all --all-features -- -D warnings'
            }
        }
        stage('test') {
            steps {                  
                sh 'cargo test --verbose --all --all-features'
            }
        }
        stage('test_coverage') {
          steps {
            sh './ci/coverage.sh'
          }
        }
    }
}
