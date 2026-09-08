TEMPLATE = app
CONFIG  += c++2b console
CONFIG  -= app_bundle
QT      += core gui widgets

TARGET = genc3wbtest

INCLUDEPATH    += $$PWD/../src
LIBS           += -L$$OUT_PWD/../src -lgenc3wb
PRE_TARGETDEPS += $$OUT_PWD/../src/libgenc3wb.a

HEADERS += neatest.h
HEADERS += test.h
HEADERS += test_widget.h
HEADERS += test_settings.h
HEADERS += test_mainwindow.h

SOURCES += neatest.cpp
SOURCES += main.cpp
SOURCES += test.cpp
SOURCES += test_widget.cpp
SOURCES += test_settings.cpp
SOURCES += test_mainwindow.cpp
