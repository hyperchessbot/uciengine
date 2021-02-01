call meta\b

python meta\gen.py

git add . -A

git commit -m "%*"

pause

git push origin master

git push gitlab master
