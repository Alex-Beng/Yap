import pickle
import json

root_paths = [
    'dumps/',
    'text_dumps/'
]

for rp in root_paths:
    x = pickle.load(open(f'{rp}x.pk', 'rb'))
    y = pickle.load(open(f'{rp}y.pk', 'rb'))
    assert(len(x) == len(y))
    # 适应旧版标注
    for i in range(len(x)):
        x[i] = f'{rp}{x[i]}_raw.jpg'
    json.dump(x, open(f'{rp}x.json', 'w', encoding='utf-8'), ensure_ascii=False)
    json.dump(y, open(f'{rp}y.json', 'w', encoding='utf-8'), ensure_ascii=False)
    # 再读取
    # x = json.load(open(f'{rp}x.json', 'r'))