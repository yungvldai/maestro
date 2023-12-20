### Changes:

- no requirement for readiness probe if app has dependents
- no separate thread for handling signals
- using libc calls instead of separate processes (kill & id) (#4)
- some refactoring