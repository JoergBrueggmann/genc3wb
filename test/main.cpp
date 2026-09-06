/**
 * @file      main.cpp
 * @brief     The entry point of the test suite.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "test.h"

#include <QApplication>
#include <QDir>
#include <QTemporaryDir>

/**
 * @brief   Runs the test suite in a directory of its own.
 * @details * The settings of product are stored relative to the working directory, so the suite runs in a
 *            temporary one, and a run leaves nothing behind for the next.
 * @return  0 where every test group held, 1 otherwise
 */
int main(int argc, char* argv[])
{
    QApplication    application(argc, argv);
    QTemporaryDir   dirOfRun;
    bool            bHeld = false;

    if ( ! dirOfRun.isValid() ) {
        return 1;
    }
    QDir::setCurrent(dirOfRun.path());
    bHeld = test();
    QDir::setCurrent(QDir::tempPath());
    return bHeld ? 0 : 1;
}
