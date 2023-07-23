import os
import pickle

def exist_or_create_pk(path):
    if not os.path.exists(path):
        pickle.dump([], open(path, 'wb'))

root_paths = [
    'dumps/',
    'text_dumps/'
]

xx = []
yy = []

for rp in root_paths:
    x = pickle.load(open(f'{rp}x.pk', 'rb'))
    y = pickle.load(open(f'{rp}y.pk', 'rb'))
    assert(len(x) == len(y))

    for c in x:
        xx.append(f'{rp}{c}_raw.jpg')
    yy += y

print(xx[:10])
print(yy[:10])
print(xx[-10:])
print(yy[-10:])
assert(len(xx) == len(yy))
pickle.dump(xx, open("xx.pk", 'wb'))
pickle.dump(yy, open("yy.pk", 'wb'))
