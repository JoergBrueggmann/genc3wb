TEMPLATE = lib
CONFIG  += staticlib c++2b
QT      += core gui widgets

TARGET  = genc3wb
DESTDIR = $$OUT_PWD

HEADERS += gc3lineedit.h
HEADERS += gc3plaintextedit.h
HEADERS += gc3codeeditorwidget.h
HEADERS += gc3multistatelabel.h
HEADERS += gc3processingstatelabel.h
HEADERS += gc3timerwatchdog.h
HEADERS += mdlsettings.h
HEADERS += gwbinfodialog.h
HEADERS += gwbmainwindow.h

SOURCES += gc3lineedit.cpp
SOURCES += gc3plaintextedit.cpp
SOURCES += gc3codeeditorwidget.cpp
SOURCES += gc3multistatelabel.cpp
SOURCES += gc3processingstatelabel.cpp
SOURCES += gc3timerwatchdog.cpp
SOURCES += mdlsettings.cpp
SOURCES += gwbinfodialog.cpp
SOURCES += gwbmainwindow.cpp

RESOURCES += ../resources/resources.qrc
