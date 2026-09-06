/**
 * @file      main.cpp
 * @brief     The entry point of the application.
 * @copyright (c) Jorg Karl-Heinz Walter Bruggmann, 2021-2026
 * @author    Jorg Karl-Heinz Walter Bruggmann <info@joerg-brueggmann.de>
 */

#include <QApplication>
#include <QMainWindow>

// realises FR-001
int main(int argc, char* argv[])
{
    QApplication    application(argc, argv);
    QMainWindow     window;

    window.setWindowTitle(QStringLiteral("genc3wb"));
    window.resize(800, 600);
    window.show();
    return application.exec();
}
