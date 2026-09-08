/**
 * @file      gwbinfodialog.cpp
 * @brief     The modal dialog that states what product is.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gwbinfodialog.h"

#include <QLabel>
#include <QVBoxLayout>

namespace genc3wb::mainwindow
{

GwbInfoDialog::GwbInfoDialog(QWidget* pwdgtParent) : QDialog(pwdgtParent)
{
    QVBoxLayout*    playout = new QVBoxLayout(this);
    QLabel*         plabelName = new QLabel(QStringLiteral("genc³wb"), this);
    QLabel*         plabelText = new QLabel(this);

    setObjectName(QStringLiteral("InfoDialog"));
    setWindowTitle(QStringLiteral("About genc³wb"));
    setModal(true);

    plabelName->setObjectName(QStringLiteral("labelName"));
    plabelText->setObjectName(QStringLiteral("labelText"));
    plabelText->setText(
        QStringLiteral("The Integrated Development Environment for the genc³ build environment.\n\n")
        + QStringLiteral("Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026\n")
        + QStringLiteral("See the file LICENSE for the terms under which it is provided."));
    plabelText->setWordWrap(true);

    m_pwdgtButtonBox = new QDialogButtonBox(QDialogButtonBox::Ok, this);
    m_pwdgtButtonBox->setObjectName(QStringLiteral("buttonBox"));

    playout->addWidget(plabelName);
    playout->addWidget(plabelText);
    playout->addWidget(m_pwdgtButtonBox);

    connect(m_pwdgtButtonBox, &QDialogButtonBox::accepted, this, &QDialog::accept);
    connect(m_pwdgtButtonBox, &QDialogButtonBox::rejected, this, &QDialog::reject);
}

GwbInfoDialog::~GwbInfoDialog()
{
}

QDialogButtonBox* GwbInfoDialog::pwdgtButtonBox() const
{
    return m_pwdgtButtonBox;
}

}
