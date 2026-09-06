/**
 * @file      test_widget.cpp
 * @brief     The test group of the component genc3wb::widget.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "test_widget.h"

#include "gc3codeeditorwidget.h"
#include "gc3lineedit.h"
#include "gc3multistatelabel.h"
#include "gc3plaintextedit.h"
#include "gc3processingstatelabel.h"
#include "gc3timerwatchdog.h"

#include <QFile>
#include <QPixmap>

using namespace genc3wb::widget;

/********** forward declarations **************************************************************************************/

TEST_DEF( widgetMultiStateLabelSelectsState );
TEST_DEF( widgetMultiStateLabelRejectsStateOutOfRange );
TEST_DEF( widgetProcessingStateLabelShowsFourStates );
TEST_DEF( widgetProcessingStateLabelImagesResolve );
TEST_DEF( widgetProcessingStateLabelKeepsStateSet );
TEST_DEF( widgetLineEditOmitsChangeOfProgram );
TEST_DEF( widgetLineEditReportsChangeOfUser );
TEST_DEF( widgetPlainTextEditShowsNullAsWatermark );
TEST_DEF( widgetPlainTextEditShowsTextAsText );
TEST_DEF( widgetCodeEditorNumbersItsLines );
TEST_DEF( widgetCodeEditorReadsAndWritesAStream );
TEST_DEF( widgetTimerWatchdogCountsFromAlive );

/********** test group ************************************************************************************************/

/*  * validated        : ✅
    * completeness     : ✅
    * independence     : ✅
    * edge cases       : ✅
    * conforms to doc  : ✅ */
TEST_DEF( widget )
{
    TEST_INIT();
    TEST( widgetMultiStateLabelSelectsState );
    TEST( widgetMultiStateLabelRejectsStateOutOfRange );
    TEST( widgetProcessingStateLabelShowsFourStates );
    TEST( widgetProcessingStateLabelImagesResolve );
    TEST( widgetProcessingStateLabelKeepsStateSet );
    TEST( widgetLineEditOmitsChangeOfProgram );
    TEST( widgetLineEditReportsChangeOfUser );
    TEST( widgetPlainTextEditShowsNullAsWatermark );
    TEST( widgetPlainTextEditShowsTextAsText );
    TEST( widgetCodeEditorNumbersItsLines );
    TEST( widgetCodeEditorReadsAndWritesAStream );
    TEST( widgetTimerWatchdogCountsFromAlive );
    TEST_RETURN();
}

/********** test cases ************************************************************************************************/

TEST_DEF( widgetMultiStateLabelSelectsState )
{
    TEST_INIT();

    Gc3MultiStateLabel  wdgtLabel(QStringList{"QLabel { color: red; }", "QLabel { color: green; }"});

    TEST_ASSERT( wdgtLabel.nStateCount() == 2 );
    TEST_ASSERT( wdgtLabel.nState() == -1 );
    TEST_ASSERT( wdgtLabel.setState(1) );
    TEST_ASSERT( wdgtLabel.nState() == 1 );
    TEST_ASSERT( wdgtLabel.setState(0) );
    TEST_ASSERT( wdgtLabel.nState() == 0 );
    TEST_RETURN();
}

TEST_DEF( widgetMultiStateLabelRejectsStateOutOfRange )
{
    TEST_INIT();

    Gc3MultiStateLabel  wdgtLabel(QStringList{"QLabel { color: red; }", "QLabel { color: green; }"});

    TEST_ASSERT( wdgtLabel.setState(0) );
    TEST_ASSERT( ! wdgtLabel.setState(-1) );
    TEST_ASSERT( ! wdgtLabel.setState(2) );
    TEST_ASSERT( wdgtLabel.nState() == 0 );
    TEST_RETURN();
}

TEST_DEF( widgetProcessingStateLabelShowsFourStates )
{
    TEST_INIT();

    Gc3ProcessingStateLabel wdgtIndicator;

    TEST_ASSERT( wdgtIndicator.nStateCount() == 4 );
    TEST_RETURN();
}

TEST_DEF( widgetProcessingStateLabelImagesResolve )
{
    TEST_INIT();

    // FR-017: the four states are shown as four distinct images. A style sheet
    // naming a resource that does not resolve draws nothing and reports nothing,
    // so the resources are asserted here rather than trusted.
    const char* const   aPaths[] = {
        ":/icons/icons/indicatorUntouched.png",
        ":/icons/icons/indicatorChanged.png",
        ":/icons/icons/indicatorUnknownFileUntouched.png",
        ":/icons/icons/indicatorUnknownFileChanged.png"
    };

    for ( const char* strPath : aPaths ) {
        TEST_ASSERT( QFile::exists(QString::fromLatin1(strPath)) );
        TEST_ASSERT( ! QPixmap(QString::fromLatin1(strPath)).isNull() );
    }
    TEST_RETURN();
}

TEST_DEF( widgetProcessingStateLabelKeepsStateSet )
{
    TEST_INIT();

    Gc3ProcessingStateLabel wdgtIndicator;

    wdgtIndicator.setProcessingState(ProcessingState::ValidFileTextUntouched);
    TEST_ASSERT( wdgtIndicator.eProcessingState() == ProcessingState::ValidFileTextUntouched );
    wdgtIndicator.setProcessingState(ProcessingState::UnknownFileTextChanged);
    TEST_ASSERT( wdgtIndicator.eProcessingState() == ProcessingState::UnknownFileTextChanged );
    wdgtIndicator.setProcessingState(ProcessingState::ValidFileTextChanged);
    TEST_ASSERT( wdgtIndicator.eProcessingState() == ProcessingState::ValidFileTextChanged );
    wdgtIndicator.setProcessingState(ProcessingState::UnknownFileTextUntouched);
    TEST_ASSERT( wdgtIndicator.eProcessingState() == ProcessingState::UnknownFileTextUntouched );
    TEST_RETURN();
}

TEST_DEF( widgetLineEditOmitsChangeOfProgram )
{
    TEST_INIT();

    Gc3LineEdit wdgtFileName;

    wdgtFileName.setText(QStringLiteral("some/path"));
    TEST_ASSERT( wdgtFileName.text() == QStringLiteral("some/path") );
    TEST_ASSERT( wdgtFileName.isChangeToBeOmitted() );
    TEST_ASSERT( ! wdgtFileName.isChangeToBeOmitted() );
    TEST_RETURN();
}

TEST_DEF( widgetLineEditReportsChangeOfUser )
{
    TEST_INIT();

    Gc3LineEdit wdgtFileName;

    TEST_ASSERT( ! wdgtFileName.isChangeToBeOmitted() );
    wdgtFileName.insert(QStringLiteral("typed"));
    TEST_ASSERT( wdgtFileName.text() == QStringLiteral("typed") );
    TEST_ASSERT( ! wdgtFileName.isChangeToBeOmitted() );
    TEST_RETURN();
}

TEST_DEF( widgetPlainTextEditShowsNullAsWatermark )
{
    TEST_INIT();

    Gc3PlainTextEdit    wdgtEdit;

    wdgtEdit.setPlainText(QString());
    TEST_ASSERT( wdgtEdit.isWatermarkEnabled() );
    TEST_ASSERT( wdgtEdit.strWatermark() == QStringLiteral("- null -") );
    TEST_ASSERT( wdgtEdit.toPlainText().isEmpty() );
    TEST_RETURN();
}

TEST_DEF( widgetPlainTextEditShowsTextAsText )
{
    TEST_INIT();

    Gc3PlainTextEdit    wdgtEdit;

    wdgtEdit.setPlainText(QStringLiteral("a line"));
    TEST_ASSERT( ! wdgtEdit.isWatermarkEnabled() );
    TEST_ASSERT( wdgtEdit.toPlainText() == QStringLiteral("a line") );
    wdgtEdit.setPlainText(QStringLiteral(""));
    TEST_ASSERT( ! wdgtEdit.isWatermarkEnabled() );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorNumbersItsLines )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    int                 nWidthOfOneDigit = 0;

    wdgtEditor.setPlainText(QStringLiteral("one"));
    nWidthOfOneDigit = wdgtEditor.nLineNumberAreaWidth();
    TEST_ASSERT( nWidthOfOneDigit > 0 );
    TEST_ASSERT( wdgtEditor.blockCount() == 1 );

    wdgtEditor.setPlainText(QStringLiteral("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11"));
    TEST_ASSERT( wdgtEditor.blockCount() == 11 );
    TEST_ASSERT( wdgtEditor.nLineNumberAreaWidth() > nWidthOfOneDigit );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorReadsAndWritesAStream )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    QString             strRead = QStringLiteral("first\nsecond");
    QString             strWritten;
    QTextStream         streamIn(&strRead, QIODevice::ReadOnly);
    QTextStream         streamOut(&strWritten, QIODevice::WriteOnly);

    wdgtEditor.readStream(streamIn);
    TEST_ASSERT( wdgtEditor.toPlainText() == strRead );
    TEST_ASSERT( wdgtEditor.isChangeToBeOmitted() );

    wdgtEditor.saveToStream(streamOut);
    streamOut.flush();
    TEST_ASSERT( strWritten == strRead );

    wdgtEditor.empty();
    TEST_ASSERT( wdgtEditor.toPlainText().isEmpty() );
    TEST_RETURN();
}

TEST_DEF( widgetTimerWatchdogCountsFromAlive )
{
    TEST_INIT();

    Gc3TimerWatchdog    wdgtWatchdog(10, 3, false);

    TEST_ASSERT( ! wdgtWatchdog.isEnabled() );
    TEST_ASSERT( wdgtWatchdog.nTicksNoKeepAlive() == 0 );

    wdgtWatchdog.alive();
    TEST_ASSERT( wdgtWatchdog.isEnabled() );
    TEST_ASSERT( wdgtWatchdog.nTicksNoKeepAlive() == 0 );

    wdgtWatchdog.suspend();
    TEST_ASSERT( ! wdgtWatchdog.isActive() );
    wdgtWatchdog.resume();
    TEST_ASSERT( wdgtWatchdog.isActive() );
    TEST_RETURN();
}
