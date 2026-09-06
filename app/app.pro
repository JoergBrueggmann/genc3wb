TEMPLATE = app
CONFIG  += c++2b
CONFIG  -= console
QT      += core gui widgets

TARGET = genc3wb

INCLUDEPATH += $$PWD/../src
LIBS        += -L$$OUT_PWD/../src -lgenc3wb
PRE_TARGETDEPS += $$OUT_PWD/../src/libgenc3wb.a

SOURCES += main.cpp
