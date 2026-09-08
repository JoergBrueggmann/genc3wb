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

#include <QElapsedTimer>
#include <QEventLoop>
#include <QFile>
#include <QPixmap>
#include <QTemporaryDir>
#include <QTimer>

#include <cstdio>

// The capture of standard output (widgetCodeEditorWritesIncrementAsUtf8) needs the file descriptor of a
// stream. The functions are those of POSIX, which Microsoft's runtime carries under its own names.
#ifdef _WIN32
#include <io.h>
#define TEST_DUP        _dup
#define TEST_DUP2       _dup2
#define TEST_CLOSE      _close
#define TEST_FILENO     _fileno
#else
#include <unistd.h>
#define TEST_DUP        dup
#define TEST_DUP2       dup2
#define TEST_CLOSE      close
#define TEST_FILENO     fileno
#endif

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
TEST_DEF( widgetCodeEditorProvidesWholeTextAsFirstIncrement );
TEST_DEF( widgetCodeEditorProvidesInsertionAsIncrement );
TEST_DEF( widgetCodeEditorProvidesDeletionAsIncrement );
TEST_DEF( widgetCodeEditorProvidesReplacementAsIncrement );
TEST_DEF( widgetCodeEditorProvidesNoIncrementWithoutModification );
TEST_DEF( widgetCodeEditorCarriesChangeOfProgramIntoIncrement );
TEST_DEF( widgetCodeEditorComposesTheWholeTextOfItsIncrements );
TEST_DEF( widgetCodeEditorRendersIncrementAsOneLine );
TEST_DEF( widgetCodeEditorKeepsMaxIdleTimeSet );
TEST_DEF( widgetCodeEditorProvidesIncrementOnIdle );
TEST_DEF( widgetCodeEditorDerivesIncrementOfOneMegabyteInTime );
TEST_DEF( widgetCodeEditorWritesIncrementAsUtf8 );

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
    TEST( widgetCodeEditorProvidesWholeTextAsFirstIncrement );
    TEST( widgetCodeEditorProvidesInsertionAsIncrement );
    TEST( widgetCodeEditorProvidesDeletionAsIncrement );
    TEST( widgetCodeEditorProvidesReplacementAsIncrement );
    TEST( widgetCodeEditorProvidesNoIncrementWithoutModification );
    TEST( widgetCodeEditorCarriesChangeOfProgramIntoIncrement );
    TEST( widgetCodeEditorComposesTheWholeTextOfItsIncrements );
    TEST( widgetCodeEditorRendersIncrementAsOneLine );
    TEST( widgetCodeEditorKeepsMaxIdleTimeSet );
    TEST( widgetCodeEditorProvidesIncrementOnIdle );
    TEST( widgetCodeEditorDerivesIncrementOfOneMegabyteInTime );
    TEST( widgetCodeEditorWritesIncrementAsUtf8 );
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

/**
 * @brief   Yields the text that results from overwriting the range of the increment at its position.
 * @details * This is the operation a receiver of the increments carries out, written here so that the
 *            increments are asserted against what a receiver would reconstruct from them.
 * @param   strProvided     the text the receiver holds
 * @param   incr            the increment to overwrite with
 * @return  the text the receiver holds afterwards
 */
static QString strOverwrittenOfIncrement(const QString& strProvided, const TextIncrement& incr)
{
    QString     strResult = strProvided;

    strResult.replace(incr.nPosition, incr.nRange, incr.strText);
    return strResult;
}

TEST_DEF( widgetCodeEditorProvidesWholeTextAsFirstIncrement )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    TextIncrement       incr;

    // FR-053: the provided text is empty before the first increment
    TEST_ASSERT( wdgtEditor.strTextProvided().isEmpty() );

    wdgtEditor.setPlainText(QStringLiteral("abc"));
    incr = wdgtEditor.incrementPending();
    TEST_ASSERT( incr.nPosition == 0 );
    TEST_ASSERT( incr.nRange == 0 );
    TEST_ASSERT( incr.strText == QStringLiteral("abc") );
    TEST_ASSERT( ! incr.isEmpty() );

    // FR-056: providing the increment makes the current text the provided text
    TEST_ASSERT( wdgtEditor.provideIncrement() );
    TEST_ASSERT( wdgtEditor.strTextProvided() == QStringLiteral("abc") );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorProvidesInsertionAsIncrement )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    TextIncrement       incr;

    wdgtEditor.setPlainText(QStringLiteral("abc"));
    TEST_ASSERT( wdgtEditor.provideIncrement() );

    wdgtEditor.setPlainText(QStringLiteral("abZc"));
    incr = wdgtEditor.incrementPending();
    // FR-050, FR-051, FR-052: an insertion overwrites no character of the provided text
    TEST_ASSERT( incr.nPosition == 2 );
    TEST_ASSERT( incr.nRange == 0 );
    TEST_ASSERT( incr.strText == QStringLiteral("Z") );
    // FR-054: overwriting the range at the position yields the current text
    TEST_ASSERT( strOverwrittenOfIncrement(wdgtEditor.strTextProvided(), incr) == QStringLiteral("abZc") );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorProvidesDeletionAsIncrement )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    TextIncrement       incr;

    wdgtEditor.setPlainText(QStringLiteral("abcd"));
    TEST_ASSERT( wdgtEditor.provideIncrement() );

    wdgtEditor.setPlainText(QStringLiteral("abd"));
    incr = wdgtEditor.incrementPending();
    // FR-050, FR-051, FR-052: a deletion carries an empty text
    TEST_ASSERT( incr.nPosition == 2 );
    TEST_ASSERT( incr.nRange == 1 );
    TEST_ASSERT( incr.strText.isEmpty() );
    TEST_ASSERT( ! incr.isEmpty() );
    // FR-054
    TEST_ASSERT( strOverwrittenOfIncrement(wdgtEditor.strTextProvided(), incr) == QStringLiteral("abd") );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorProvidesReplacementAsIncrement )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    TextIncrement       incr;

    wdgtEditor.setPlainText(QStringLiteral("abcdef"));
    TEST_ASSERT( wdgtEditor.provideIncrement() );

    wdgtEditor.setPlainText(QStringLiteral("abXYef"));
    incr = wdgtEditor.incrementPending();
    // FR-050, FR-051, FR-052
    TEST_ASSERT( incr.nPosition == 2 );
    TEST_ASSERT( incr.nRange == 2 );
    TEST_ASSERT( incr.strText == QStringLiteral("XY") );
    // FR-054
    TEST_ASSERT( strOverwrittenOfIncrement(wdgtEditor.strTextProvided(), incr) == QStringLiteral("abXYef") );

    // the whole text emptied is one increment overwriting all of it
    TEST_ASSERT( wdgtEditor.provideIncrement() );
    wdgtEditor.empty();
    incr = wdgtEditor.incrementPending();
    TEST_ASSERT( incr.nPosition == 0 );
    TEST_ASSERT( incr.nRange == 6 );
    TEST_ASSERT( incr.strText.isEmpty() );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorProvidesNoIncrementWithoutModification )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;

    // FR-057: an editor never modified has nothing to provide
    TEST_ASSERT( wdgtEditor.incrementPending().isEmpty() );
    TEST_ASSERT( ! wdgtEditor.provideIncrement() );

    wdgtEditor.setPlainText(QStringLiteral("abc"));
    TEST_ASSERT( wdgtEditor.provideIncrement() );

    // FR-057: the same text again differs from the provided text in nothing
    wdgtEditor.setPlainText(QStringLiteral("abc"));
    TEST_ASSERT( wdgtEditor.incrementPending().isEmpty() );
    TEST_ASSERT( ! wdgtEditor.provideIncrement() );
    TEST_ASSERT( wdgtEditor.strTextProvided() == QStringLiteral("abc") );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorCarriesChangeOfProgramIntoIncrement )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    QString             strRead = QStringLiteral("first\nsecond");
    QTextStream         streamIn(&strRead, QIODevice::ReadOnly);
    TextIncrement       incr;

    // FR-055: a modification made by the product, not by the user, is carried into the increment as well
    wdgtEditor.readStream(streamIn);
    TEST_ASSERT( wdgtEditor.isChangeToBeOmitted() );
    incr = wdgtEditor.incrementPending();
    TEST_ASSERT( incr.nPosition == 0 );
    TEST_ASSERT( incr.nRange == 0 );
    TEST_ASSERT( incr.strText == strRead );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorComposesTheWholeTextOfItsIncrements )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    const QString       aTexts[] = { QStringLiteral("alpha"),
                                     QStringLiteral("alpha beta"),
                                     QStringLiteral("alpha gamma beta"),
                                     QStringLiteral("gamma beta"),
                                     QStringLiteral("") };
    QString             strReceiver;

    // FR-054, FR-056: a receiver that overwrites every increment holds what the editor holds
    for ( const QString& strText : aTexts ) {
        wdgtEditor.setPlainText(strText);
        strReceiver = strOverwrittenOfIncrement(strReceiver, wdgtEditor.incrementPending());
        wdgtEditor.provideIncrement();
        TEST_ASSERT( strReceiver == strText );
        TEST_ASSERT( wdgtEditor.strTextProvided() == strText );
    }
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorRendersIncrementAsOneLine )
{
    TEST_INIT();

    // FR-060, FR-061: the numbers are padded on the left with zeros to four digits
    TEST_ASSERT( (TextIncrement{ 2, 2, QStringLiteral("XY") }).strRendering()
                 == QStringLiteral("Position:0002, Range: 0002, \"XY\"") );
    TEST_ASSERT( (TextIncrement{ 0, 0, QStringLiteral("a") }).strRendering()
                 == QStringLiteral("Position:0000, Range: 0000, \"a\"") );
    // FR-061: a number of more than four digits is written in full
    TEST_ASSERT( (TextIncrement{ 12345, 67890, QString() }).strRendering()
                 == QStringLiteral("Position:12345, Range: 67890, \"\"") );
    // FR-062: the escape sequences keep the rendering one line
    TEST_ASSERT( (TextIncrement{ 1, 0, QStringLiteral("a\nb\r\tc\"d\\e") }).strRendering()
                 == QStringLiteral("Position:0001, Range: 0000, \"a\\nb\\r\\tc\\\"d\\\\e\"") );
    TEST_ASSERT( ! (TextIncrement{ 1, 0, QStringLiteral("a\nb") }).strRendering().contains(QLatin1Char('\n')) );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorKeepsMaxIdleTimeSet )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;

    // FR-058: the maximum idle time is a number of milliseconds
    TEST_ASSERT( wdgtEditor.nMaxIdleTime() == Gc3CodeEditorWidget::nDefaultMaxIdleTime );
    wdgtEditor.setMaxIdleTime(250);
    TEST_ASSERT( wdgtEditor.nMaxIdleTime() == 250 );
    // a time below one millisecond is taken as one millisecond
    wdgtEditor.setMaxIdleTime(0);
    TEST_ASSERT( wdgtEditor.nMaxIdleTime() == 1 );
    wdgtEditor.setMaxIdleTime(-5);
    TEST_ASSERT( wdgtEditor.nMaxIdleTime() == 1 );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorProvidesIncrementOnIdle )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    TextIncrement       incrReceived;
    bool                bAnnounced = false;
    QEventLoop          loopOfWait;
    QElapsedTimer       timerOfWait;

    wdgtEditor.setMaxIdleTime(200);
    QObject::connect(&wdgtEditor, &Gc3CodeEditorWidget::incrementProvided,
                     [&incrReceived, &bAnnounced, &loopOfWait](const TextIncrement& incr) {
                         incrReceived = incr;
                         bAnnounced = true;
                         loopOfWait.quit();
                     });
    wdgtEditor.setPlainText(QStringLiteral("typed"));

    // FR-056: the increment is not provided while the maximum idle time has not elapsed
    TEST_ASSERT( ! bAnnounced );
    TEST_ASSERT( wdgtEditor.strTextProvided().isEmpty() );

    // FR-056, FR-059: it is provided and announced once the text was not modified for that time
    timerOfWait.start();
    QTimer::singleShot(10000, &loopOfWait, &QEventLoop::quit);
    loopOfWait.exec();
    TEST_ASSERT( bAnnounced );
    TEST_ASSERT( incrReceived.nPosition == 0 );
    TEST_ASSERT( incrReceived.nRange == 0 );
    TEST_ASSERT( incrReceived.strText == QStringLiteral("typed") );
    TEST_ASSERT( wdgtEditor.strTextProvided() == QStringLiteral("typed") );
    // the wait ended on the watchdog, not on the deadline of ten seconds
    TEST_ASSERT( timerOfWait.elapsed() < 5000 );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorDerivesIncrementOfOneMegabyteInTime )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    QString             strOneMegabyte;
    TextIncrement       incr;
    QElapsedTimer       timerOfDerivation;
    qint64              nMilliSecondsTaken = 0;

    for ( int nLine = 0; nLine < 1024; nLine++ ) {
        strOneMegabyte += QString(1023, QLatin1Char('x'));
        strOneMegabyte += QLatin1Char('\n');
    }
    TEST_ASSERT( strOneMegabyte.length() == (1024 * 1024) );

    wdgtEditor.setPlainText(strOneMegabyte);
    TEST_ASSERT( wdgtEditor.provideIncrement() );
    TEST_ASSERT( wdgtEditor.strTextProvided().length() == (1024 * 1024) );

    // NFR-005: one character changed in the middle of a megabyte is derived within 200 milliseconds
    strOneMegabyte.replace(512 * 1024, 1, QLatin1Char('y'));
    wdgtEditor.setPlainText(strOneMegabyte);
    timerOfDerivation.start();
    incr = wdgtEditor.incrementPending();
    nMilliSecondsTaken = timerOfDerivation.elapsed();
    TEST_ASSERT( incr.nPosition == (512 * 1024) );
    TEST_ASSERT( incr.nRange == 1 );
    TEST_ASSERT( incr.strText == QStringLiteral("y") );
    TEST_ASSERT( nMilliSecondsTaken < 200 );
    TEST_RETURN();
}

TEST_DEF( widgetCodeEditorWritesIncrementAsUtf8 )
{
    TEST_INIT();

    Gc3CodeEditorWidget wdgtEditor;
    QTemporaryDir       dirOfCapture;
    QString             strPathOfCapture;
    QFile               fileOfCapture;
    QByteArray          bytesWritten;
    const QString       strNonAscii = QString::fromUtf8("\xc3\xa4\xe2\x82\xac");
    const QByteArray    bytesExpected = QByteArray("Position:0000, Range: 0000, \"\xc3\xa4\xe2\x82\xac\"\n");
    int                 nStdOutSaved = -1;

    TEST_ASSERT( dirOfCapture.isValid() );
    strPathOfCapture = dirOfCapture.filePath(QStringLiteral("stdout.txt"));

    // IR-015, IR-016: the bytes standard output carries are read back rather than trusted, so that the
    // encoding is asserted where it takes effect and not where it is configured.
    wdgtEditor.setPlainText(strNonAscii);
    fflush(stdout);
    nStdOutSaved = TEST_DUP(TEST_FILENO(stdout));
    TEST_ASSERT( nStdOutSaved >= 0 );
    if ( freopen(strPathOfCapture.toLocal8Bit().constData(), "wb", stdout) != nullptr ) {
        wdgtEditor.provideIncrement();
        fflush(stdout);
    }
    TEST_DUP2(nStdOutSaved, TEST_FILENO(stdout));
    TEST_CLOSE(nStdOutSaved);

    fileOfCapture.setFileName(strPathOfCapture);
    TEST_ASSERT( fileOfCapture.open(QIODevice::ReadOnly) );
    bytesWritten = fileOfCapture.readAll();
    fileOfCapture.close();

    TEST_ASSERT( bytesWritten == bytesExpected );
    // the two characters are carried as their UTF-8 sequences and not as one byte each
    TEST_ASSERT( bytesWritten.contains(QByteArray("\xc3\xa4")) );
    TEST_ASSERT( bytesWritten.contains(QByteArray("\xe2\x82\xac")) );
    TEST_ASSERT( QString::fromUtf8(bytesWritten).contains(strNonAscii) );
    TEST_RETURN();
}
