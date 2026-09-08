/****************************************************************************
**
** Copyright (C) 2015 The Qt Company Ltd.
** Contact: http://www.qt.io/licensing/
**
** This file is part of the examples of the Qt Toolkit.
**
** $QT_BEGIN_LICENSE:BSD$
** You may use this file under the terms of the BSD license as follows:
**
** "Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions are
** met:
**   * Redistributions of source code must retain the above copyright
**     notice, this list of conditions and the following disclaimer.
**   * Redistributions in binary form must reproduce the above copyright
**     notice, this list of conditions and the following disclaimer in
**     the documentation and/or other materials provided with the
**     distribution.
**   * Neither the name of The Qt Company Ltd nor the names of its
**     contributors may be used to endorse or promote products derived
**     from this software without specific prior written permission.
**
**
** THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
** "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
** LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
** A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
** OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
** SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
** LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
** DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
** THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
** (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
** OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."
**
** $QT_END_LICENSE$
**
****************************************************************************/

/**
 * @file      gc3codeeditorwidget.h
 * @brief     A code editor with a line number area and a highlighted current line.
 */

#ifndef GC3CODEEDITORWIDGET_H
#define GC3CODEEDITORWIDGET_H

#include "gc3plaintextedit.h"
#include "gc3timerwatchdog.h"

#include <QPaintEvent>
#include <QResizeEvent>
#include <QSize>
#include <QString>
#include <QTextStream>
#include <QWidget>

namespace genc3wb::widget
{

class LineNumberArea;

// realises FR-050, FR-051, FR-052, FR-057, FR-060, FR-061, FR-062
/**
 * @brief   How the text of a code editor changed since the text increment before it.
 * @details * Overwriting nRange characters of the provided text at nPosition with strText yields the text
 *            the code editor currently holds.
 *          * A pure insertion carries nRange 0, a pure deletion an empty strText.
 *          * An increment describing no change carries nPosition, nRange 0 and an empty strText.
 * @par prefix incr
 */
struct TextIncrement
{
    int     nPosition = 0;  ///< the characters of the provided text preceding where strText overwrites
    int     nRange    = 0;  ///< the characters of the provided text that strText overwrites, from nPosition
    QString strText;        ///< the text overwriting that range

    // realises FR-057
    /**
     * @brief   Yields whether the increment describes no change of the provided text.
     * @return  whether it overwrites no character and inserts no text
     */
    bool isEmpty() const;

    // realises FR-060, FR-061, FR-062
    /**
     * @brief   Yields the increment as one line.
     * @details * The form is 'Position:<PPPP>, Range: <NNNN>, "<Text>"'.
     *          * The position and the range are written as decimal numbers of at least four digits, padded
     *            on the left with the digit zero.
     *          * The text carries a backslash, a double quote, a line feed, a carriage return and a
     *            horizontal tab as the escape sequences of C, so that the rendering stays one line.
     * @return  the rendering of the increment
     */
    QString strRendering() const;
};

// realises FR-008, FR-009, FR-010
/**
 * @brief   A code editor that numbers its lines and highlights the line the cursor is in.
 * @details * Carried over from an example of the Qt Company; the licence notice above holds for this file.
 *          * The current line is highlighted only while the editor holds the focus.
 * @par prefix wdgt
 */
class Gc3CodeEditorWidget : public Gc3PlainTextEdit
{
    Q_OBJECT

    friend class LineNumberArea;

public: // constants
    static const int nDefaultMaxIdleTime = 1000;    ///< the maximum idle time until another one is set, in milliseconds
    static const int nTicksOfMaxIdleTime = 10;      ///< the ticks into which the maximum idle time is divided

public: // constructors / destructors
    explicit Gc3CodeEditorWidget(QWidget* pwdgtParent = nullptr);
    virtual ~Gc3CodeEditorWidget();

public: // methods
    /** @brief Sets whether the line the cursor is in is highlighted. */
    virtual void setup(bool bHighlightingOfCurrentLine);

    /** @brief Empties the document. */
    virtual void empty();

    /** @brief Reads the whole stream into the document. */
    virtual void readStream(QTextStream& stream);

    /** @brief Writes the document into the stream. */
    virtual void saveToStream(QTextStream& stream);

    /**
     * @brief   Yields whether the next change event is to be omitted, and clears the mark.
     * @return  whether the change was caused by the program rather than by the user
     */
    virtual bool isChangeToBeOmitted();

    /** @brief Yields the width of the line number area, in pixels. */
    virtual int nLineNumberAreaWidth() const;

    // increments

    // realises FR-058
    /**
     * @brief   Sets the maximum idle time and configures the watchdog accordingly.
     * @details * The watchdog is given a tick of nMilliSecondsMaxIdleTime divided by nTicksOfMaxIdleTime, and a
     *            threshold of nTicksOfMaxIdleTime - 1, so that an increment is provided at most one maximum
     *            idle time after the last modification of the text.
     *          * A time below 1 millisecond is taken as 1 millisecond.
     *          * The ticks counted so far are discarded.
     * @param   nMilliSecondsMaxIdleTime    the maximum idle time, in milliseconds
     */
    virtual void setMaxIdleTime(int nMilliSecondsMaxIdleTime);

    // realises FR-058
    /**
     * @brief   Yields the maximum idle time.
     * @return  the maximum idle time, in milliseconds
     */
    virtual int nMaxIdleTime() const;

    // realises FR-053, FR-056
    /**
     * @brief   Yields the provided text: the text every increment provided so far composes.
     * @return  the text a receiver of the increments holds, empty before the first increment was provided
     */
    virtual const QString& strTextProvided() const;

    // realises FR-054, FR-055
    /**
     * @brief   Derives the increment from the provided text to the text the editor currently holds.
     * @details * Overwriting the range of the result at its position with its text yields the current text.
     *          * The editor is not changed; the increment is only derived.
     * @return  the increment, an empty one where the current text does not differ from the provided text
     */
    virtual TextIncrement incrementPending() const;

    // realises FR-056, FR-057, FR-059, IR-015, IR-016
    /**
     * @brief   Provides the pending increment: writes it to standard output and announces it.
     * @details * The provided text becomes the text the editor currently holds.
     *          * An empty increment is neither written nor announced, and the provided text stays as it is.
     *          * What is written is encoded in UTF-8, set on the stream rather than left at the encoding
     *            the framework defaults to.
     * @return  whether an increment was provided
     */
    virtual bool provideIncrement();

public: // overridden functions
    virtual void setPlainText(const QString& strText) override;
    virtual void mousePressEvent(QMouseEvent* pEvent) override;

signals:
    // realises FR-059
    /** @brief Announces the increment that the code editor has just provided. */
    void incrementProvided(const genc3wb::widget::TextIncrement& incr);

public slots:
    /** @brief Answers a change of the focus, so that the highlight follows it. */
    void on_focusChanged(bool bReceivedFocusAndNotLost);

protected slots:
    void updateLineNumberAreaWidth(int nNewBlockCount);
    void highlightCurrentLine();
    void on_updateRequest(const QRect& rect, int nDy);

    // realises FR-055
    /** @brief Answers a modification of the text, whatever caused it, by keeping the watchdog alive. */
    void on_textModified();

    // realises FR-056
    /** @brief Answers the expiry of the maximum idle time by providing the pending increment. */
    void on_maxIdleTimeExpired();

protected: // internal methods
    virtual void lineNumberAreaPaintEvent(QPaintEvent* pEvent);
    virtual void resizeEvent(QResizeEvent* pEvent) override;

protected: // attributes
    QWidget*    m_pwdgtLineNumberArea;          ///< the area that draws the line numbers
    bool        m_bOmitChangeEvent;             ///< whether the next change event was caused by the program
    bool        m_bHighlightingOfCurrentLine;   ///< whether the current line is highlighted
    bool        m_bHasFocus;                    ///< whether the editor holds the focus

    // increments
    Gc3TimerWatchdog*   m_pWatchdog;            ///< expires where the text was not modified for the maximum idle time
    QString             m_strTextProvided;      ///< the text every increment provided so far composes
    int                 m_nMaxIdleTime;         ///< the maximum idle time, in milliseconds
};

/**
 * @brief   The area at the left edge of a code editor that draws the line numbers.
 * @par prefix wdgt
 */
class LineNumberArea : public QWidget
{
    Q_OBJECT

public: // constructors / destructors
    explicit LineNumberArea(Gc3CodeEditorWidget* pwdgtEditor);
    virtual ~LineNumberArea();

public: // overridden functions
    virtual QSize sizeHint() const override;

protected: // overridden functions
    virtual void paintEvent(QPaintEvent* pEvent) override;

protected: // attributes
    Gc3CodeEditorWidget*    m_pwdgtCodeEditor;  ///< the editor this area belongs to
};

}

#endif // GC3CODEEDITORWIDGET_H
