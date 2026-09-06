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

#include <QPaintEvent>
#include <QResizeEvent>
#include <QSize>
#include <QTextStream>
#include <QWidget>

namespace genc3wb::widget
{

class LineNumberArea;

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

public: // overridden functions
    virtual void setPlainText(const QString& strText);
    virtual void mousePressEvent(QMouseEvent* pEvent) override;

public slots:
    /** @brief Answers a change of the focus, so that the highlight follows it. */
    void on_focusChanged(bool bReceivedFocusAndNotLost);

protected slots:
    void updateLineNumberAreaWidth(int nNewBlockCount);
    void highlightCurrentLine();
    void on_updateRequest(const QRect& rect, int nDy);

protected: // internal methods
    virtual void lineNumberAreaPaintEvent(QPaintEvent* pEvent);
    virtual void resizeEvent(QResizeEvent* pEvent) override;

protected: // attributes
    QWidget*    m_pwdgtLineNumberArea;          ///< the area that draws the line numbers
    bool        m_bOmitChangeEvent;             ///< whether the next change event was caused by the program
    bool        m_bHighlightingOfCurrentLine;   ///< whether the current line is highlighted
    bool        m_bHasFocus;                    ///< whether the editor holds the focus
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
