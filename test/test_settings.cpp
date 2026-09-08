/**
 * @file      test_settings.cpp
 * @brief     The test group of the component genc3wb::settings.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "test_settings.h"

#include "mdlsettings.h"

using namespace genc3wb::settings;

/********** forward declarations **************************************************************************************/

TEST_DEF( settingsHoldsPathOfEachInputKind );
TEST_DEF( settingsHoldsPathOfCompilerCompiler );
TEST_DEF( settingsHoldsPathOfOutFileWithinRange );
TEST_DEF( settingsRejectsOutFileNumberOutOfRange );
TEST_DEF( settingsExcludesOutFile );
TEST_DEF( settingsSurvivesSaveAndLoad );

/********** test group ************************************************************************************************/

/*  * validated        : ✅
    * completeness     : ✅
    * independence     : ✅
    * edge cases       : ✅
    * conforms to doc  : ✅ */
TEST_DEF( settings )
{
    TEST_INIT();
    TEST( settingsHoldsPathOfEachInputKind );
    TEST( settingsHoldsPathOfCompilerCompiler );
    TEST( settingsHoldsPathOfOutFileWithinRange );
    TEST( settingsRejectsOutFileNumberOutOfRange );
    TEST( settingsExcludesOutFile );
    TEST( settingsSurvivesSaveAndLoad );
    TEST_RETURN();
}

/********** test cases ************************************************************************************************/

TEST_DEF( settingsHoldsPathOfEachInputKind )
{
    TEST_INIT();

    MdlSettings stgsSettings;

    stgsSettings.setInpFilePath(InputKind::CompilerCompilerInput, QStringLiteral("cc.genc3"));
    stgsSettings.setInpFilePath(InputKind::CompilerInput, QStringLiteral("c.txt"));
    TEST_ASSERT( stgsSettings.strInpFilePathOfKind(InputKind::CompilerCompilerInput) == QStringLiteral("cc.genc3") );
    TEST_ASSERT( stgsSettings.strInpFilePathOfKind(InputKind::CompilerInput) == QStringLiteral("c.txt") );
    TEST_RETURN();
}

TEST_DEF( settingsHoldsPathOfCompilerCompiler )
{
    TEST_INIT();

    MdlSettings stgsSettings;

    TEST_ASSERT( ! stgsSettings.strCCExecFilePath().isEmpty() );
    stgsSettings.setCCExecFilePath(QStringLiteral("tools/genc3"));
    TEST_ASSERT( stgsSettings.strCCExecFilePath() == QStringLiteral("tools/genc3") );
    TEST_RETURN();
}

TEST_DEF( settingsHoldsPathOfOutFileWithinRange )
{
    TEST_INIT();

    MdlSettings stgsSettings;
    QString     strOutFilePath;

    stgsSettings.excludeOutFilePath(MdlSettings::nMinFileNumber);
    stgsSettings.excludeOutFilePath(MdlSettings::nMaxFileNumber);
    TEST_ASSERT( ! stgsSettings.bOutFilePathOfNumber(MdlSettings::nMinFileNumber, strOutFilePath) );
    TEST_ASSERT( stgsSettings.setOutFilePath(MdlSettings::nMinFileNumber, QStringLiteral("out1.txt")) );
    TEST_ASSERT( stgsSettings.bOutFilePathOfNumber(MdlSettings::nMinFileNumber, strOutFilePath) );
    TEST_ASSERT( strOutFilePath == QStringLiteral("out1.txt") );

    TEST_ASSERT( stgsSettings.setOutFilePath(MdlSettings::nMaxFileNumber, QStringLiteral("out9.txt")) );
    TEST_ASSERT( stgsSettings.bOutFilePathOfNumber(MdlSettings::nMaxFileNumber, strOutFilePath) );
    TEST_ASSERT( strOutFilePath == QStringLiteral("out9.txt") );
    TEST_RETURN();
}

TEST_DEF( settingsRejectsOutFileNumberOutOfRange )
{
    TEST_INIT();

    MdlSettings stgsSettings;
    QString     strOutFilePath;

    TEST_ASSERT( ! stgsSettings.setOutFilePath(MdlSettings::nMinFileNumber - 1, QStringLiteral("below.txt")) );
    TEST_ASSERT( ! stgsSettings.setOutFilePath(MdlSettings::nMaxFileNumber + 1, QStringLiteral("above.txt")) );
    TEST_ASSERT( ! stgsSettings.bOutFilePathOfNumber(MdlSettings::nMinFileNumber - 1, strOutFilePath) );
    TEST_ASSERT( ! stgsSettings.bOutFilePathOfNumber(MdlSettings::nMaxFileNumber + 1, strOutFilePath) );
    TEST_RETURN();
}

TEST_DEF( settingsExcludesOutFile )
{
    TEST_INIT();

    MdlSettings stgsSettings;
    QString     strOutFilePath;

    TEST_ASSERT( stgsSettings.setOutFilePath(3, QStringLiteral("out3.txt")) );
    TEST_ASSERT( stgsSettings.bOutFilePathOfNumber(3, strOutFilePath) );
    stgsSettings.excludeOutFilePath(3);
    TEST_ASSERT( ! stgsSettings.bOutFilePathOfNumber(3, strOutFilePath) );
    TEST_RETURN();
}

TEST_DEF( settingsSurvivesSaveAndLoad )
{
    TEST_INIT();

    QString strOutFilePath;

    {
        MdlSettings stgsWritten;

        stgsWritten.setInpFilePath(InputKind::CompilerCompilerInput, QStringLiteral("kept-cc.genc3"));
        stgsWritten.setInpFilePath(InputKind::CompilerInput, QStringLiteral("kept-c.txt"));
        stgsWritten.setCCExecFilePath(QStringLiteral("kept-genc3"));
        stgsWritten.setOutFilePath(2, QStringLiteral("kept-out2.txt"));
        stgsWritten.save();
    }

    MdlSettings stgsRead;

    TEST_ASSERT( stgsRead.load() );
    TEST_ASSERT( stgsRead.strInpFilePathOfKind(InputKind::CompilerCompilerInput) == QStringLiteral("kept-cc.genc3") );
    TEST_ASSERT( stgsRead.strInpFilePathOfKind(InputKind::CompilerInput) == QStringLiteral("kept-c.txt") );
    TEST_ASSERT( stgsRead.strCCExecFilePath() == QStringLiteral("kept-genc3") );
    TEST_ASSERT( stgsRead.bOutFilePathOfNumber(2, strOutFilePath) );
    TEST_ASSERT( strOutFilePath == QStringLiteral("kept-out2.txt") );
    TEST_RETURN();
}
