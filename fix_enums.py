from collections import Counter
from pymongo import MongoClient

URI = "mongodb://root:Kl5QJXvNLtf6N7BfmFenRrmcAN61n4h04vcHrXq3IZFUD8zKq6AXlKuJDCuhXUl9@1.34.172.117:27017/?directConnection=true"
DB = "teamder"

# collection -> field -> (valid values, explicit renames for invalid -> valid)
SPEC = {
    "users": {
        "availability": ({"open_for_collab", "busy", "unavailable"}, {"open": "open_for_collab"}),
        "work_mode":    ({"remote", "hybrid", "in_person"}, {}),
    },
    "competitions": {
        "status":         ({"open", "closing_soon", "upcoming", "past"}, {}),
        "publish_status": ({"draft", "pending_review", "published", "rejected"}, {}),
    },
    "projects": {
        "status": ({"recruiting", "active", "completed", "archived"}, {}),
    },
    "study_groups": {
        "status": ({"recruiting", "active", "completed", "archived"}, {}),
    },
}

def target_for(value, valid, renames):
    if value in renames:
        return renames[value]
    norm = value.replace("-", "_")  # hyphen -> underscore normalization
    if norm in valid:
        return norm
    return None  # cannot safely map

c = MongoClient(URI, serverSelectionTimeoutMS=8000)
db = c[DB]
print("Connected. Collections:", db.list_collection_names(), "\n")

unmapped = []
for coll, fields in SPEC.items():
    if coll not in db.list_collection_names():
        continue
    for field, (valid, renames) in fields.items():
        counter = Counter()
        for d in db[coll].find({}, {field: 1}):
            v = d.get(field)
            if v is not None:
                counter[v] += 1
        invalid = {v: n for v, n in counter.items() if v not in valid}
        if not invalid:
            continue
        print(f"[{coll}.{field}] invalid values found: {invalid}")
        for bad, n in invalid.items():
            tgt = target_for(bad, valid, renames)
            if tgt is None:
                print(f"    !! cannot map {bad!r} ({n} docs) -- needs manual review")
                unmapped.append((coll, field, bad, n))
                continue
            res = db[coll].update_many({field: bad}, {"$set": {field: tgt}})
            print(f"    fixed {bad!r} -> {tgt!r} : {res.modified_count} docs updated")
    print()

print("DONE.")
if unmapped:
    print("UNMAPPED (left as-is):", unmapped)
