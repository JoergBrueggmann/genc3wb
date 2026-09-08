TEMPLATE = subdirs
CONFIG  += ordered

SUBDIRS += src
SUBDIRS += app
SUBDIRS += test

app.depends  = src
test.depends = src
