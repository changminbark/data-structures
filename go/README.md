# Go
### Without benchstat (manual comparison)
```go
go test -bench=. > old.txt
```
Make some changes...
```go
go test -bench=. > new.txt
```
Now you manually compare old.txt vs new.txt

### With benchstat (automatic comparison)
```bash
benchstat old.txt new.txt
```
Shows statistical significance of changes