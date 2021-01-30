python meta\gen.py

call meta\b

call cargo run --example example

pause

git add . -A

git commit -m "%*"

pause

git push origin master

git push gitlab master

pause

cargo publish
