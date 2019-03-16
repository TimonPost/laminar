pipeline {
    agent none
    stages {
        stage('Test on Windows') {
            agent { 
                label 'windows' 
            }
            steps {
                bat 'cargo test'
            }
        }
    }
}