/**
 * @file      gc3processingstatelabel.cpp
 * @brief     The indicator of the processing state of an input group.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gc3processingstatelabel.h"

namespace genc3wb::widget
{

namespace
{

const QStringList l_strListStyleSheets =
{
    "QLabel { background-image: url(:/icons/icons/indicatorUntouched.png); }",
    "QLabel { background-image: url(:/icons/icons/indicatorChanged.png); }",
    "QLabel { background-image: url(:/icons/icons/indicatorUnknownFileUntouched.png); }",
    "QLabel { background-image: url(:/icons/icons/indicatorUnknownFileChanged.png); }"
};

}

Gc3ProcessingStateLabel::Gc3ProcessingStateLabel(QWidget* pwdgtParent) :
    Gc3MultiStateLabel(l_strListStyleSheets, pwdgtParent)
{
    setProcessingState(ProcessingState::UnknownFileTextUntouched);
}

Gc3ProcessingStateLabel::~Gc3ProcessingStateLabel()
{
}

void Gc3ProcessingStateLabel::setProcessingState(ProcessingState eState)
{
    setState(int(eState));
}

ProcessingState Gc3ProcessingStateLabel::eProcessingState() const
{
    int     nStateShown = nState();

    return (nStateShown < 0) ? ProcessingState::ValidFileTextUntouched : ProcessingState(nStateShown);
}

}
