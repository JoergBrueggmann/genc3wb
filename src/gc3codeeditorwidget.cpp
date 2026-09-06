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
 * @file      gc3codeeditorwidget.cpp
 * @brief     A code editor with a line number area and a highlighted current line.
 */

#include "gc3codeeditorwidget.h"

#include <QPainter>
#include <QTextBlock>

namespace genc3wb::widget
{

Gc3CodeEditorWidget::Gc3CodeEditorWidget(QWidget* pwdgtParent) : Gc3PlainTextEdit(pwdgtParent)
{
    m_pwdgtLineNumberArea = new LineNumberArea(this);
    m_bOmitChangeEvent = false;
    m_bHighlightingOfCurrentLine = false;
    m_bHasFocus = false;
    connect(this, &QPlainTextEdit::blockCountChanged, this, &Gc3CodeEditorWidget::updateLineNumberAreaWidth);
    connect(this, &QPlainTextEdit::updateRequest, this, &Gc3CodeEditorWidget::on_updateRequest);
    updateLineNumberAreaWidth(0);
}

Gc3CodeEditorWidget::~Gc3CodeEditorWidget()
{
}

void Gc3CodeEditorWidget::setup(bool bHighlightingOfCurrentLine)
{
    m_bHighlightingOfCurrentLine = bHighlightingOfCurrentLine;
    connect(this, &QPlainTextEdit::cursorPositionChanged, this, &Gc3CodeEditorWidget::highlightCurrentLine);
    if ( m_bHighlightingOfCurrentLine && m_bHasFocus ) {
        highlightCurrentLine();
    }
}

void Gc3CodeEditorWidget::empty()
{
    setPlainText("");
}

void Gc3CodeEditorWidget::readStream(QTextStream& stream)
{
    setPlainText(stream.readAll());
}

void Gc3CodeEditorWidget::saveToStream(QTextStream& stream)
{
    stream << toPlainText();
}

bool Gc3CodeEditorWidget::isChangeToBeOmitted()
{
    bool    bChangeToBeOmitted = m_bOmitChangeEvent;

    m_bOmitChangeEvent = false;
    return bChangeToBeOmitted;
}

int Gc3CodeEditorWidget::nLineNumberAreaWidth() const
{
    int     nDigits = 1;
    int     nMax = qMax(1, blockCount());

    while ( nMax >= 10 ) {
        nMax /= 10;
        ++nDigits;
    }
    return 3 + (fontMetrics().horizontalAdvance(QLatin1Char('9')) * nDigits);
}

void Gc3CodeEditorWidget::updateLineNumberAreaWidth(int)
{
    setViewportMargins(nLineNumberAreaWidth(), 0, 0, 0);
}

void Gc3CodeEditorWidget::highlightCurrentLine()
{
    if ( m_bHighlightingOfCurrentLine ) {
        QList<QTextEdit::ExtraSelection> listExtraSelections;

        if ( m_bHasFocus ) {
            QTextEdit::ExtraSelection   selection;

            selection.format.setBackground(QColor(Qt::yellow).lighter(160));
            selection.format.setProperty(QTextFormat::FullWidthSelection, true);
            selection.cursor = textCursor();
            selection.cursor.clearSelection();
            listExtraSelections.append(selection);
        }
        setExtraSelections(listExtraSelections);
    }
}

void Gc3CodeEditorWidget::on_updateRequest(const QRect& rect, int nDy)
{
    if ( nDy ) {
        m_pwdgtLineNumberArea->scroll(0, nDy);
    } else {
        m_pwdgtLineNumberArea->update(0, rect.y(), m_pwdgtLineNumberArea->width(), rect.height());
    }
    if ( rect.contains(viewport()->rect()) ) {
        updateLineNumberAreaWidth(0);
    }
}

void Gc3CodeEditorWidget::on_focusChanged(bool bReceivedFocusAndNotLost)
{
    m_bHasFocus = bReceivedFocusAndNotLost;
    highlightCurrentLine();
}

void Gc3CodeEditorWidget::setPlainText(const QString& strText)
{
    m_bOmitChangeEvent = true;
    Gc3PlainTextEdit::setPlainText(strText);
}

void Gc3CodeEditorWidget::mousePressEvent(QMouseEvent* pEvent)
{
    Gc3PlainTextEdit::mousePressEvent(pEvent);
    highlightCurrentLine();
}

void Gc3CodeEditorWidget::lineNumberAreaPaintEvent(QPaintEvent* pEvent)
{
    QPainter    painter(m_pwdgtLineNumberArea);
    QTextBlock  block = firstVisibleBlock();
    int         nBlockNumber = block.blockNumber();
    int         nTop = int(blockBoundingGeometry(block).translated(contentOffset()).top());
    int         nBottom = nTop + int(blockBoundingRect(block).height());

    painter.fillRect(pEvent->rect(), Qt::lightGray);
    while ( block.isValid() && (nTop <= (pEvent->rect().bottom())) ) {
        if ( block.isVisible() && (nBottom >= (pEvent->rect().top())) ) {
            painter.setPen(Qt::black);
            painter.drawText(0, nTop, m_pwdgtLineNumberArea->width(), fontMetrics().height(),
                             Qt::AlignRight, QString::number(nBlockNumber + 1));
        }
        block = block.next();
        nTop = nBottom;
        nBottom = nTop + int(blockBoundingRect(block).height());
        ++nBlockNumber;
    }
}

void Gc3CodeEditorWidget::resizeEvent(QResizeEvent* pEvent)
{
    Gc3PlainTextEdit::resizeEvent(pEvent);

    QRect   rectContents = contentsRect();

    m_pwdgtLineNumberArea->setGeometry(
        QRect(rectContents.left(), rectContents.top(), nLineNumberAreaWidth(), rectContents.height()));
}

LineNumberArea::LineNumberArea(Gc3CodeEditorWidget* pwdgtEditor) : QWidget(pwdgtEditor)
{
    m_pwdgtCodeEditor = pwdgtEditor;
}

LineNumberArea::~LineNumberArea()
{
}

QSize LineNumberArea::sizeHint() const
{
    return QSize(m_pwdgtCodeEditor->nLineNumberAreaWidth(), 0);
}

void LineNumberArea::paintEvent(QPaintEvent* pEvent)
{
    m_pwdgtCodeEditor->lineNumberAreaPaintEvent(pEvent);
}

}
