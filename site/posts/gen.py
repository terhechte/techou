template = """
[frontMatter]
title = "%s"
tags = ["first tag", "second tag"]
created = "2019-01-%s"
description = "A run around the world"
published = true
---
# test
this is the actual article contents yeah.
## test2
this is the actual article contents yeah.
"""
words = filter(lambda x: len(x) > 1, map(lambda x: x.strip(), "You missed something  Velcro is loud The US Army removed velcro from their uniforms around 2010 since the opening of flaps gave positions away  Edit To the  replies saying they still have velcro".split(" ")))
import random
print random.choice(words)
day = 30
for article in range(0, 15):
    mw = [random.choice(words).strip() for word in range(0, random.randint(2,5))]
    title = " ".join(mw)
    print title.strip()
    contents = template % (title, day)
    day -= 1
    filename = "-".join(mw).strip()
    filename = filename.lower() + ".md"
    print filename
    fp = open(filename, "w")
    fp.write(contents)
    fp.close()


