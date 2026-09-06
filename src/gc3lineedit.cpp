/**
 * @file      gc3lineedit.cpp
 * @brief     A line edit that shows whether it is read-only and can suppress its change event.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gc3lineedit.h"

#include <QEvent>
#include <QPalette>

namespace genc3wb::widget
{

Gc3LineEdit::Gc3LineEdit(QWidget* pwdgtParent) : QLineEdit(pwdgtParent)
{
    m_bOmitChangeEvent = false;
}

Gc3LineEdit::Gc3LineEdit(const QString& strContents, QWidget* pwdgtParent) : QLineEdit(strContents, pwdgtParent)
{
    m_bOmitChangeEvent = false;
}

Gc3LineEdit::~Gc3LineEdit()
{
}

bool Gc3LineEdit::event(QEvent* pEvent)
{
    if ( (pEvent->type()) == QEvent::Polish ) {
        updateColor(isReadOnly());
    }
    return QLineEdit::event(pEvent);
}

void Gc3LineEdit::setReadOnly(bool bReadOnly)
{
    updateColor(bReadOnly);
    QLineEdit::setReadOnly(bReadOnly);
}

void Gc3LineEdit::setText(const QString& strText)
{
    m_bOmitChangeEvent = true;
    QLineEdit::setText(strText);
}

bool Gc3LineEdit::isChangeToBeOmitted()
{
    bool    bChangeToBeOmitted = m_bOmitChangeEvent;

    m_bOmitChangeEvent = false;
    return bChangeToBeOmitted;
}

void Gc3LineEdit::updateColor(bool bReadOnly)
{
    QPalette    paletteOfState = palette();

    if ( bReadOnly ) {
        paletteOfState.setColor(QPalette::Base, paletteOfState.color(QPalette::Window));
    } else {
        paletteOfState.setColor(QPalette::Base, paletteOfState.color(QPalette::Light));
    }
    setPalette(paletteOfState);
}

}
