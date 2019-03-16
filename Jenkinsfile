pipeline {
    agent none
    stages {
        stage('Test on Windows') {
            agent { 
                label 'windows' 
            }
            steps {
                bat 'C:\Users\root\.cargo\bin\cargo test'
            }
        }
    }
}
