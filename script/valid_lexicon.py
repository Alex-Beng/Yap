import json
from PIL import Image

all_name = json.load(open('./models/all_list.json', 'r', encoding='utf-8'))

lexicon = set({})
for name in all_name:
    for char in name:
        lexicon.add(char)

lexicon = sorted(list(lexicon))

x = json.load(open('./xx.json', 'r', encoding='utf-8'))
y = json.load(open('./yy.json', 'r', encoding='utf-8'))

not_in_lexicon = set({})
for i, lb in enumerate(y):

    for c in lb:
        if c not in lexicon:
            print(lb)
            # img = Image.open(x[i])
            # img.show()
            
            not_in_lexicon.add(c)
            break
print(not_in_lexicon)
