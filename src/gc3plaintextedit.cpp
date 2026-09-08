/**
 * @file      gc3plaintextedit.cpp
 * @brief     A plain text edit that can show a watermark across its viewport.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gc3plaintextedit.h"

#include <QPainter>
#include <QRect>
#include <QtMath>

namespace genc3wb::widget
{

Gc3PlainTextEdit::Gc3PlainTextEdit(QWidget* pwdgtParent) : QPlainTextEdit(pwdgtParent)
{
    m_bEnable = false;
}

Gc3PlainTextEdit::~Gc3PlainTextEdit()
{
}

void Gc3PlainTextEdit::setWatermark(const QString& strWatermark, bool bEnable)
{
    m_strWatermark = strWatermark;
    m_bEnable = bEnable;
}

void Gc3PlainTextEdit::enableWatermark(bool bEnable)
{
    m_bEnable = bEnable;
}

const QString& Gc3PlainTextEdit::strWatermark() const
{
    return m_strWatermark;
}

bool Gc3PlainTextEdit::isWatermarkEnabled() const
{
    return m_bEnable;
}

void Gc3PlainTextEdit::setNull(bool bNull)
{
    if ( bNull ) {
        setWatermark("- null -");
        QPlainTextEdit::setPlainText("");
    } else {
        enableWatermark(false);
    }
}

void Gc3PlainTextEdit::setPlainText(const QString& strText)
{
    if ( strText.isNull() ) {
        setNull();
    } else {
        setNull(false);
        QPlainTextEdit::setPlainText(strText);
    }
}

void Gc3PlainTextEdit::paintEvent(QPaintEvent* pEvent)
{
    if ( m_bEnable && ( ! m_strWatermark.isEmpty()) ) {
        QPainter        painterText(viewport());
        QFontMetrics    fontMetricsOfText(painterText.font());
        qreal           qrWidth = width();
        qreal           qrHeight = height();
        qreal           qrRotationAngle = qRadiansToDegrees(qAtan2(qrWidth, qrHeight) - (M_PI / 2.0));
        qreal           qrMarginFactor = 0.3;
        qreal           qrDiagonal = qSqrt( (qrWidth * qrWidth) + (qrHeight * qrHeight) );
        qreal           qrInnerDiagonal = qrDiagonal * (1.0 - (2.0 * qrMarginFactor));
        qreal           qrAdvance = fontMetricsOfText.horizontalAdvance(m_strWatermark);
        qreal           qrScale = (qrAdvance > 0.0) ? (qrInnerDiagonal / qrAdvance) : 1.0;
        qreal           qrMargin = qrDiagonal * qrMarginFactor;

        painterText.setPen(Qt::lightGray);
        painterText.translate(0, qrHeight);
        painterText.rotate(qrRotationAngle);
        painterText.scale(qrScale, qrScale);
        painterText.translate(qrMargin / qrScale, 0);
        painterText.translate(0, qreal(fontMetricsOfText.height()) * 1.1 / qrScale);
        painterText.drawText(0, 0, m_strWatermark);
    }
    QPlainTextEdit::paintEvent(pEvent);
}

}
