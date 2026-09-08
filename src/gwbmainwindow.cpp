/**
 * @file      gwbmainwindow.cpp
 * @brief     The main window of the workbench.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gwbmainwindow.h"

#include "gwbinfodialog.h"

#include <QGridLayout>
#include <QHBoxLayout>
#include <QMenuBar>
#include <QStatusBar>
#include <QVBoxLayout>
#include <QWidget>

namespace genc3wb::mainwindow
{

using genc3wb::settings::InputKind;
using genc3wb::widget::Gc3CodeEditorWidget;
using genc3wb::widget::Gc3LineEdit;
using genc3wb::widget::Gc3ProcessingStateLabel;
using genc3wb::widget::ProcessingState;

namespace
{

const int nIndicatorEdge = 24;  ///< the edge length of an indicator, in pixels

}

GwbMainWindow::GwbMainWindow(QWidget* pwdgtParent) : QMainWindow(pwdgtParent)
{
    QWidget*        pwdgtCentral = new QWidget(this);
    QGridLayout*    playout = new QGridLayout(pwdgtCentral);

    setObjectName(QStringLiteral("MainWindow"));
    setWindowTitle(QStringLiteral("genc³wb"));
    pwdgtCentral->setObjectName(QStringLiteral("centralwidget"));

    m_groupCCInp = buildInputGroup(QStringLiteral("Compiler-compiler input"),
                                   QStringLiteral("CCInp"), pwdgtCentral);
    m_groupCInp  = buildInputGroup(QStringLiteral("Compiler input"),
                                   QStringLiteral("CInp"), pwdgtCentral);
    buildCompilerCompilerGroup(pwdgtCentral);
    buildOutputGroup(pwdgtCentral);

    // The two input groups stand on the left, the compiler-compiler and the
    // output on the right, as the layout sketch of the design states it.
    playout->addWidget(m_groupCCInp.pwdgtGroupBox, 0, 0);
    playout->addWidget(m_groupCInp.pwdgtGroupBox,  1, 0);
    playout->addWidget(m_pwdgtGroupBoxCOut,        0, 1);
    playout->addWidget(m_pwdgtGroupBoxCC,          1, 1);
    playout->setColumnStretch(0, 1);
    playout->setColumnStretch(1, 1);
    playout->setRowStretch(0, 1);
    playout->setRowStretch(1, 1);

    setCentralWidget(pwdgtCentral);
    buildMenuAndStatusBar();
    resize(1100, 750);
}

GwbMainWindow::~GwbMainWindow()
{
}

InputGroup GwbMainWindow::buildInputGroup(const QString& strTitle, const QString& strSuffix, QWidget* pwdgtParent)
{
    InputGroup      group;
    QVBoxLayout*    playoutGroup = nullptr;
    QHBoxLayout*    playoutHead = nullptr;

    group.pwdgtGroupBox = new QGroupBox(strTitle, pwdgtParent);
    group.pwdgtGroupBox->setObjectName(QStringLiteral("groupBox") + strSuffix);

    playoutGroup = new QVBoxLayout(group.pwdgtGroupBox);
    playoutHead = new QHBoxLayout();

    group.pwdgtFileName = new Gc3LineEdit(group.pwdgtGroupBox);
    group.pwdgtFileName->setObjectName(QStringLiteral("lineEdit") + strSuffix);
    group.pwdgtFileName->setPlaceholderText(QStringLiteral("path of the file"));

    group.pwdgtFileSelector = new QToolButton(group.pwdgtGroupBox);
    group.pwdgtFileSelector->setObjectName(QStringLiteral("toolButton") + strSuffix);
    group.pwdgtFileSelector->setText(QStringLiteral("..."));

    group.pwdgtIndicator = new Gc3ProcessingStateLabel(group.pwdgtGroupBox);
    group.pwdgtIndicator->setObjectName(QStringLiteral("labelIndicator") + strSuffix);
    group.pwdgtIndicator->setFixedSize(nIndicatorEdge, nIndicatorEdge);
    group.pwdgtIndicator->setProcessingState(ProcessingState::UnknownFileTextUntouched);

    group.pwdgtCodeEditor = new Gc3CodeEditorWidget(group.pwdgtGroupBox);
    group.pwdgtCodeEditor->setObjectName(QStringLiteral("plainTextEdit") + strSuffix);
    group.pwdgtCodeEditor->setup(true);

    playoutHead->addWidget(group.pwdgtFileName);
    playoutHead->addWidget(group.pwdgtFileSelector);
    playoutHead->addWidget(group.pwdgtIndicator);

    playoutGroup->addLayout(playoutHead);
    playoutGroup->addWidget(group.pwdgtCodeEditor);

    return group;
}

void GwbMainWindow::buildCompilerCompilerGroup(QWidget* pwdgtParent)
{
    QHBoxLayout*    playout = nullptr;

    m_pwdgtGroupBoxCC = new QGroupBox(QStringLiteral("Compiler-compiler"), pwdgtParent);
    m_pwdgtGroupBoxCC->setObjectName(QStringLiteral("groupBoxCC"));

    playout = new QHBoxLayout(m_pwdgtGroupBoxCC);

    m_pwdgtLineEditCC = new Gc3LineEdit(m_pwdgtGroupBoxCC);
    m_pwdgtLineEditCC->setObjectName(QStringLiteral("lineEditCC"));
    m_pwdgtLineEditCC->setPlaceholderText(QStringLiteral("path of the compiler-compiler"));

    m_pwdgtToolButtonCC = new QToolButton(m_pwdgtGroupBoxCC);
    m_pwdgtToolButtonCC->setObjectName(QStringLiteral("toolButtonCC"));
    m_pwdgtToolButtonCC->setText(QStringLiteral("..."));

    m_pwdgtIndicatorCC = new Gc3ProcessingStateLabel(m_pwdgtGroupBoxCC);
    m_pwdgtIndicatorCC->setObjectName(QStringLiteral("labelIndicatorCC"));
    m_pwdgtIndicatorCC->setFixedSize(nIndicatorEdge, nIndicatorEdge);
    m_pwdgtIndicatorCC->setProcessingState(ProcessingState::UnknownFileTextUntouched);

    playout->addWidget(m_pwdgtLineEditCC);
    playout->addWidget(m_pwdgtToolButtonCC);
    playout->addWidget(m_pwdgtIndicatorCC);

    m_pwdgtGroupBoxCC->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Maximum);
}

void GwbMainWindow::buildOutputGroup(QWidget* pwdgtParent)
{
    QVBoxLayout*    playoutGroup = nullptr;
    QHBoxLayout*    playoutBar = nullptr;

    m_pwdgtGroupBoxCOut = new QGroupBox(QStringLiteral("Output"), pwdgtParent);
    m_pwdgtGroupBoxCOut->setObjectName(QStringLiteral("groupBoxCOut"));

    playoutGroup = new QVBoxLayout(m_pwdgtGroupBoxCOut);
    playoutBar = new QHBoxLayout();

    m_pwdgtPushButtonLeft = new QPushButton(QStringLiteral("<"), m_pwdgtGroupBoxCOut);
    m_pwdgtPushButtonLeft->setObjectName(QStringLiteral("pushButtonLeft"));

    m_pwdgtLabelPage = new QLabel(m_pwdgtGroupBoxCOut);
    m_pwdgtLabelPage->setObjectName(QStringLiteral("labelPage"));
    m_pwdgtLabelPage->setAlignment(Qt::AlignCenter);

    m_pwdgtPushButtonRight = new QPushButton(QStringLiteral(">"), m_pwdgtGroupBoxCOut);
    m_pwdgtPushButtonRight->setObjectName(QStringLiteral("pushButtonRight"));

    m_pwdgtPushButtonCOut = new QPushButton(QStringLiteral("Detach"), m_pwdgtGroupBoxCOut);
    m_pwdgtPushButtonCOut->setObjectName(QStringLiteral("pushButtonCOut"));

    m_pwdgtStackedWidget = new QStackedWidget(m_pwdgtGroupBoxCOut);
    m_pwdgtStackedWidget->setObjectName(QStringLiteral("stackedWidget"));

    playoutBar->addWidget(m_pwdgtPushButtonLeft);
    playoutBar->addWidget(m_pwdgtLabelPage);
    playoutBar->addWidget(m_pwdgtPushButtonRight);
    playoutBar->addStretch(1);
    playoutBar->addWidget(m_pwdgtPushButtonCOut);

    playoutGroup->addLayout(playoutBar);
    playoutGroup->addWidget(m_pwdgtStackedWidget);

    // The pages themselves belong to the component genc3wb::output; the group
    // carries the stacked widget that will hold them, and states that meanwhile.
    m_pwdgtLabelPage->setText(QStringLiteral("- no page -"));
    m_pwdgtPushButtonLeft->setEnabled(false);
    m_pwdgtPushButtonRight->setEnabled(false);
    m_pwdgtPushButtonCOut->setEnabled(false);
}

void GwbMainWindow::buildMenuAndStatusBar()
{
    QMenuBar*   pwdgtMenuBar = menuBar();
    QAction*    pactionInfo = nullptr;

    pwdgtMenuBar->setObjectName(QStringLiteral("menubar"));

    m_pwdgtMenuHelp = pwdgtMenuBar->addMenu(QStringLiteral("&Help"));
    m_pwdgtMenuHelp->setObjectName(QStringLiteral("menuHelp"));

    pactionInfo = m_pwdgtMenuHelp->addAction(QStringLiteral("&Info"));
    pactionInfo->setObjectName(QStringLiteral("actionInfo"));
    connect(pactionInfo, &QAction::triggered, this, &GwbMainWindow::on_actionInfo_triggered);

    statusBar()->setObjectName(QStringLiteral("statusbar"));
    statusBar()->showMessage(QStringLiteral("Ready"));
}

const InputGroup& GwbMainWindow::groupOfKind(InputKind eKind) const
{
    return (eKind == InputKind::CompilerCompilerInput) ? m_groupCCInp : m_groupCInp;
}

QGroupBox* GwbMainWindow::pwdgtGroupBoxCC() const
{
    return m_pwdgtGroupBoxCC;
}

QGroupBox* GwbMainWindow::pwdgtGroupBoxCOut() const
{
    return m_pwdgtGroupBoxCOut;
}

QMenu* GwbMainWindow::pwdgtMenuHelp() const
{
    return m_pwdgtMenuHelp;
}

void GwbMainWindow::on_actionInfo_triggered()
{
    GwbInfoDialog   dialog(this);

    dialog.exec();
}

void GwbMainWindow::on_focusChanged(QWidget* pwdgtOld, QWidget* pwdgtNow)
{
    Gc3CodeEditorWidget* const  aEditors[] = { m_groupCCInp.pwdgtCodeEditor, m_groupCInp.pwdgtCodeEditor };

    for ( Gc3CodeEditorWidget* pwdgtEditor : aEditors ) {
        if ( pwdgtEditor == nullptr ) {
            continue;
        }
        if ( pwdgtEditor == pwdgtNow ) {
            pwdgtEditor->on_focusChanged(true);
        } else if ( pwdgtEditor == pwdgtOld ) {
            pwdgtEditor->on_focusChanged(false);
        }
    }
}

}
