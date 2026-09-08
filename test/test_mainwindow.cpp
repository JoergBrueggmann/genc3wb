/**
 * @file      test_mainwindow.cpp
 * @brief     The test group of the component genc3wb::mainwindow.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "test_mainwindow.h"

#include "gwbinfodialog.h"
#include "gwbmainwindow.h"

#include <QGroupBox>
#include <QMenuBar>
#include <QStatusBar>

using genc3wb::mainwindow::GwbInfoDialog;
using genc3wb::mainwindow::GwbMainWindow;
using genc3wb::mainwindow::InputGroup;
using genc3wb::settings::InputKind;
using genc3wb::widget::ProcessingState;

/********** forward declarations **************************************************************************************/

TEST_DEF( mainwindowComposesFourGroups );
TEST_DEF( mainwindowCarriesMenuBarWithHelpMenu );
TEST_DEF( mainwindowCarriesStatusBar );
TEST_DEF( mainwindowInputGroupCarriesItsWidgets );
TEST_DEF( mainwindowInputGroupsAreDistinct );
TEST_DEF( mainwindowCompilerCompilerGroupCarriesItsWidgets );
TEST_DEF( mainwindowOutputGroupCarriesItsWidgets );
TEST_DEF( mainwindowIndicatorShowsFourStates );
TEST_DEF( mainwindowFocusChangeReachesTheEditors );
TEST_DEF( mainwindowInfoDialogIsModalAndCloses );

/********** test group ************************************************************************************************/

/*  * validated        : ✅
    * completeness     : ✅
    * independence     : ✅
    * edge cases       : ✅
    * conforms to doc  : ✅ */
TEST_DEF( mainwindow )
{
    TEST_INIT();
    TEST( mainwindowComposesFourGroups );
    TEST( mainwindowCarriesMenuBarWithHelpMenu );
    TEST( mainwindowCarriesStatusBar );
    TEST( mainwindowInputGroupCarriesItsWidgets );
    TEST( mainwindowInputGroupsAreDistinct );
    TEST( mainwindowCompilerCompilerGroupCarriesItsWidgets );
    TEST( mainwindowOutputGroupCarriesItsWidgets );
    TEST( mainwindowIndicatorShowsFourStates );
    TEST( mainwindowFocusChangeReachesTheEditors );
    TEST( mainwindowInfoDialogIsModalAndCloses );
    TEST_RETURN();
}

/********** test cases ************************************************************************************************/

TEST_DEF( mainwindowComposesFourGroups )
{
    TEST_INIT();

    GwbMainWindow   wndMain;

    // FR-001: the four groups of the main window
    TEST_ASSERT( wndMain.findChild<QGroupBox*>(QStringLiteral("groupBoxCCInp")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QGroupBox*>(QStringLiteral("groupBoxCInp")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QGroupBox*>(QStringLiteral("groupBoxCC")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QGroupBox*>(QStringLiteral("groupBoxCOut")) != nullptr );
    TEST_ASSERT( wndMain.centralWidget() != nullptr );
    TEST_RETURN();
}

TEST_DEF( mainwindowCarriesMenuBarWithHelpMenu )
{
    TEST_INIT();

    GwbMainWindow   wndMain;

    // FR-002, FR-003: a menu bar with a help menu carrying the info entry
    TEST_ASSERT( wndMain.menuBar() != nullptr );
    TEST_ASSERT( wndMain.pwdgtMenuHelp() != nullptr );
    TEST_ASSERT( wndMain.pwdgtMenuHelp()->actions().size() == 1 );
    TEST_ASSERT( wndMain.findChild<QAction*>(QStringLiteral("actionInfo")) != nullptr );
    TEST_RETURN();
}

TEST_DEF( mainwindowCarriesStatusBar )
{
    TEST_INIT();

    GwbMainWindow   wndMain;

    // FR-005: a status bar
    TEST_ASSERT( wndMain.statusBar() != nullptr );
    TEST_ASSERT( wndMain.statusBar()->objectName() == QStringLiteral("statusbar") );
    TEST_RETURN();
}

TEST_DEF( mainwindowInputGroupCarriesItsWidgets )
{
    TEST_INIT();

    GwbMainWindow       wndMain;
    const InputGroup&   groupCCInp = wndMain.groupOfKind(InputKind::CompilerCompilerInput);

    // FR-007: file name field, file selector button, code editor, indicator
    TEST_ASSERT( groupCCInp.pwdgtGroupBox != nullptr );
    TEST_ASSERT( groupCCInp.pwdgtFileName != nullptr );
    TEST_ASSERT( groupCCInp.pwdgtFileSelector != nullptr );
    TEST_ASSERT( groupCCInp.pwdgtIndicator != nullptr );
    TEST_ASSERT( groupCCInp.pwdgtCodeEditor != nullptr );

    // FR-008: the code editor numbers its lines
    TEST_ASSERT( groupCCInp.pwdgtCodeEditor->nLineNumberAreaWidth() > 0 );
    TEST_RETURN();
}

TEST_DEF( mainwindowInputGroupsAreDistinct )
{
    TEST_INIT();

    GwbMainWindow       wndMain;
    const InputGroup&   groupCCInp = wndMain.groupOfKind(InputKind::CompilerCompilerInput);
    const InputGroup&   groupCInp = wndMain.groupOfKind(InputKind::CompilerInput);

    // FR-001: the two input groups are two, not one shown twice
    TEST_ASSERT( groupCCInp.pwdgtGroupBox != groupCInp.pwdgtGroupBox );
    TEST_ASSERT( groupCCInp.pwdgtCodeEditor != groupCInp.pwdgtCodeEditor );
    TEST_ASSERT( groupCCInp.pwdgtFileName != groupCInp.pwdgtFileName );
    TEST_ASSERT( groupCCInp.pwdgtIndicator != groupCInp.pwdgtIndicator );

    groupCCInp.pwdgtFileName->setText(QStringLiteral("cc.genc3"));
    groupCInp.pwdgtFileName->setText(QStringLiteral("c.txt"));
    TEST_ASSERT( groupCCInp.pwdgtFileName->text() == QStringLiteral("cc.genc3") );
    TEST_ASSERT( groupCInp.pwdgtFileName->text() == QStringLiteral("c.txt") );
    TEST_RETURN();
}

TEST_DEF( mainwindowCompilerCompilerGroupCarriesItsWidgets )
{
    TEST_INIT();

    GwbMainWindow   wndMain;

    // FR-022: file name field, file selector button, indicator
    TEST_ASSERT( wndMain.pwdgtGroupBoxCC() != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("lineEditCC")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("toolButtonCC")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("labelIndicatorCC")) != nullptr );
    TEST_RETURN();
}

TEST_DEF( mainwindowOutputGroupCarriesItsWidgets )
{
    TEST_INIT();

    GwbMainWindow   wndMain;

    // FR-027, FR-032 to FR-036, FR-043: the output group and its bar
    TEST_ASSERT( wndMain.pwdgtGroupBoxCOut() != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("stackedWidget")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("pushButtonLeft")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("pushButtonRight")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("labelPage")) != nullptr );
    TEST_ASSERT( wndMain.findChild<QWidget*>(QStringLiteral("pushButtonCOut")) != nullptr );
    TEST_RETURN();
}

TEST_DEF( mainwindowIndicatorShowsFourStates )
{
    TEST_INIT();

    GwbMainWindow       wndMain;
    const InputGroup&   groupCInp = wndMain.groupOfKind(InputKind::CompilerInput);

    // FR-017: four distinct images, and the state set is the state shown
    TEST_ASSERT( groupCInp.pwdgtIndicator->nStateCount() == 4 );
    groupCInp.pwdgtIndicator->setProcessingState(ProcessingState::ValidFileTextUntouched);
    TEST_ASSERT( groupCInp.pwdgtIndicator->eProcessingState() == ProcessingState::ValidFileTextUntouched );
    groupCInp.pwdgtIndicator->setProcessingState(ProcessingState::UnknownFileTextChanged);
    TEST_ASSERT( groupCInp.pwdgtIndicator->eProcessingState() == ProcessingState::UnknownFileTextChanged );
    TEST_RETURN();
}

TEST_DEF( mainwindowFocusChangeReachesTheEditors )
{
    TEST_INIT();

    GwbMainWindow       wndMain;
    const InputGroup&   groupCCInp = wndMain.groupOfKind(InputKind::CompilerCompilerInput);
    const InputGroup&   groupCInp = wndMain.groupOfKind(InputKind::CompilerInput);

    // FR-006, FR-009, FR-010: the notification reaches both editors and neither throws.
    // The highlight itself is drawn, so what is asserted here is that the editors
    // answer the notification and keep their content.
    groupCCInp.pwdgtCodeEditor->setPlainText(QStringLiteral("one\ntwo"));
    groupCInp.pwdgtCodeEditor->setPlainText(QStringLiteral("three"));

    wndMain.on_focusChanged(nullptr, groupCCInp.pwdgtCodeEditor);
    wndMain.on_focusChanged(groupCCInp.pwdgtCodeEditor, groupCInp.pwdgtCodeEditor);
    wndMain.on_focusChanged(groupCInp.pwdgtCodeEditor, nullptr);

    TEST_ASSERT( groupCCInp.pwdgtCodeEditor->toPlainText() == QStringLiteral("one\ntwo") );
    TEST_ASSERT( groupCInp.pwdgtCodeEditor->toPlainText() == QStringLiteral("three") );
    TEST_RETURN();
}

TEST_DEF( mainwindowInfoDialogIsModalAndCloses )
{
    TEST_INIT();

    GwbMainWindow   wndMain;
    GwbInfoDialog   dlgInfo(&wndMain);

    // FR-003, FR-004: the dialog is modal and is closed by its button box.
    // exec() is not called: it would run an event loop that nothing ends.
    TEST_ASSERT( dlgInfo.isModal() );
    TEST_ASSERT( dlgInfo.pwdgtButtonBox() != nullptr );
    TEST_ASSERT( dlgInfo.parentWidget() == &wndMain );

    dlgInfo.accept();
    TEST_ASSERT( dlgInfo.result() == QDialog::Accepted );
    TEST_RETURN();
}
