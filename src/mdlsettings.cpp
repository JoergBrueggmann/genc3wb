/**
 * @file      mdlsettings.cpp
 * @brief     The paths of product, restored between sessions.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "mdlsettings.h"

#include <QDataStream>
#include <QDir>
#include <QFile>

namespace genc3wb::settings
{

namespace
{

#if defined(_WIN32)
const QString l_strConfigFolderPath = ".\\.genc3config";
const QString l_strPathDelimiter    = "\\";
const QString l_strCCExecDefault    = ".\\genc3.exe";
#else
const QString l_strConfigFolderPath = "./.genc3config";
const QString l_strPathDelimiter    = "/";
const QString l_strCCExecDefault    = "./genc3";
#endif

const QString l_strConfigFileName = "genc3wbconfig.dat";
const QString l_strPreamble       = "ccsettings";
const qint64  l_nVersion          = 1;

QString strConfigFilePath()
{
    return l_strConfigFolderPath + l_strPathDelimiter + l_strConfigFileName;
}

}

MdlSettings::MdlSettings()
{
    if ( ! load() ) {
        m_strInpFilePath[int(InputKind::CompilerCompilerInput)] = ".";
        m_strInpFilePath[int(InputKind::CompilerInput)]         = ".";
        m_strCCExecFilePath                                     = l_strCCExecDefault;
    }
}

MdlSettings::~MdlSettings()
{
    save();
}

bool MdlSettings::load()
{
    bool    bLoaded = false;
    QFile   file(strConfigFilePath());

    if ( file.open(QIODevice::ReadOnly) ) {
        QDataStream inStream(&file);
        QString     strPreamble;
        qint64      nVersion = 0;

        inStream >> strPreamble;
        if ( strPreamble == l_strPreamble ) {
            inStream >> nVersion;
            if ( nVersion == l_nVersion ) {
                inStream >> m_strInpFilePath[int(InputKind::CompilerCompilerInput)];
                inStream >> m_strInpFilePath[int(InputKind::CompilerInput)];
                inStream >> m_strCCExecFilePath;
                inStream >> m_mapOutFilePath;
                bLoaded = true;
            }
        }
    }
    return bLoaded;
}

void MdlSettings::save()
{
    QDir    dir;

    dir.mkpath(l_strConfigFolderPath);

    QFile   file(strConfigFilePath());

    if ( file.open(QIODevice::WriteOnly) ) {
        QDataStream outStream(&file);

        outStream << l_strPreamble;
        outStream << l_nVersion;
        outStream << m_strInpFilePath[int(InputKind::CompilerCompilerInput)];
        outStream << m_strInpFilePath[int(InputKind::CompilerInput)];
        outStream << m_strCCExecFilePath;
        outStream << m_mapOutFilePath;
    }
}

const QString& MdlSettings::strInpFilePathOfKind(InputKind eKind) const
{
    return m_strInpFilePath[int(eKind)];
}

void MdlSettings::setInpFilePath(InputKind eKind, const QString& strFilePath)
{
    m_strInpFilePath[int(eKind)] = strFilePath;
}

const QString& MdlSettings::strCCExecFilePath() const
{
    return m_strCCExecFilePath;
}

void MdlSettings::setCCExecFilePath(const QString& strFilePath)
{
    m_strCCExecFilePath = strFilePath;
}

bool MdlSettings::bOutFilePathOfNumber(int nFileNumber, QString& strOutFilePath) const
{
    bool    bAvailable = false;

    if ( (nFileNumber >= nMinFileNumber) && (nFileNumber <= nMaxFileNumber) ) {
        if ( m_mapOutFilePath.contains(nFileNumber) ) {
            strOutFilePath = m_mapOutFilePath[nFileNumber];
            bAvailable = true;
        }
    }
    return bAvailable;
}

bool MdlSettings::setOutFilePath(int nFileNumber, const QString& strOutFilePath)
{
    bool    bStored = false;

    if ( (nFileNumber >= nMinFileNumber) && (nFileNumber <= nMaxFileNumber) ) {
        m_mapOutFilePath[nFileNumber] = strOutFilePath;
        bStored = true;
    }
    return bStored;
}

void MdlSettings::excludeOutFilePath(int nFileNumber)
{
    m_mapOutFilePath.remove(nFileNumber);
}

}
