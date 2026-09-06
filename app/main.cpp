/**
 * @file      main.cpp
 * @brief     The entry point of the application.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gwbmainwindow.h"

#include "gc3processingstatelabel.h"

#include <QApplication>

// realises FR-001, FR-006
int main(int argc, char* argv[])
{
    QApplication                            application(argc, argv);

    genc3wb::widget::initResources();
    genc3wb::mainwindow::GwbMainWindow      window;

    // The focus is reported by the application, not by the window, so the
    // notification of FR-006 is connected here.
    QObject::connect(&application, &QApplication::focusChanged,
                     &window, &genc3wb::mainwindow::GwbMainWindow::on_focusChanged);
    window.show();
    return application.exec();
}
