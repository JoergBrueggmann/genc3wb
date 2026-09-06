/**
 * @file      mdlsettings.h
 * @brief     The paths of product, restored between sessions.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef MDLSETTINGS_H
#define MDLSETTINGS_H

#include <QMap>
#include <QObject>
#include <QString>

namespace genc3wb::settings
{

// realises FR-001
/**
 * @brief Which of the two input files an input group edits.
 */
enum class InputKind
{
    CompilerCompilerInput = 0,  ///< the file that configures syntax and generators
    CompilerInput         = 1   ///< the file that is parsed according to that configuration
};

// realises FR-031, FR-048, FR-049
/**
 * @brief   The paths product restores between sessions.
 * @details * The paths are read at construction and written at destruction.
 *          * An output file that is not held is not presented.
 * @par prefix stgs
 */
class MdlSettings : public QObject
{
    Q_OBJECT

public: // constants
    static const int nMinFileNumber = 1;    ///< the number of the first output file
    static const int nMaxFileNumber = 9;    ///< the number of the last output file

public: // constructors / destructors
    MdlSettings();
    virtual ~MdlSettings();

public: // methods
    // realises FR-048
    /**
     * @brief   Restores the paths stored at the last termination.
     * @return  whether the stored paths could be read
     */
    virtual bool load();

    // realises FR-049
    /** @brief Stores the paths, to be restored at the next start. */
    virtual void save();

    /**
     * @brief   Yields the path of one input file.
     * @param   eKind   which input file
     * @return  the stored path, empty where none is stored
     */
    virtual const QString& strInpFilePathOfKind(InputKind eKind) const;

    // realises FR-012, FR-049
    /**
     * @brief   Sets the path of one input file.
     * @param   eKind           which input file
     * @param   strFilePath     the path to store
     */
    virtual void setInpFilePath(InputKind eKind, const QString& strFilePath);

    /**
     * @brief   Yields the path of the compiler-compiler.
     * @return  the stored path
     */
    virtual const QString& strCCExecFilePath() const;

    // realises FR-024, FR-049
    /** @brief Sets the path of the compiler-compiler. */
    virtual void setCCExecFilePath(const QString& strFilePath);

    // realises FR-031, FR-038
    /**
     * @brief   Yields the path of one output file.
     * @param   nFileNumber         the number of the output file, from nMinFileNumber to nMaxFileNumber
     * @param   strOutFilePath      receives the path
     * @return  whether an output file of that number is held
     */
    virtual bool bOutFilePathOfNumber(int nFileNumber, QString& strOutFilePath) const;

    // realises FR-038
    /**
     * @brief   Sets the path of one output file.
     * @param   nFileNumber         the number of the output file, from nMinFileNumber to nMaxFileNumber
     * @param   strOutFilePath      the path to store
     * @return  whether the number is within its range and the path was stored
     */
    virtual bool setOutFilePath(int nFileNumber, const QString& strOutFilePath);

    // realises FR-037
    /** @brief Removes the output file of that number, so that it is no longer presented. */
    virtual void excludeOutFilePath(int nFileNumber);

protected: // attributes
    QString             m_strInpFilePath[2];    ///< the path of each input file, indexed by InputKind
    QString             m_strCCExecFilePath;    ///< the path of the compiler-compiler
    QMap<int, QString>  m_mapOutFilePath;       ///< the path of each held output file, by its number
};

}

#endif // MDLSETTINGS_H
